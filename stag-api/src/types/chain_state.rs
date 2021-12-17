use std::time::Duration;

#[cfg(not(feature = "wasm"))]
use anyhow::Context;
use anyhow::{ensure, Result};
#[cfg(feature = "wasm")]
use grpc_web_client::Client;
use num_rational::Ratio;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tendermint::node::Id as NodeId;
use time::OffsetDateTime;
#[cfg(not(feature = "wasm"))]
use tonic::transport::Channel;
use url::Url;

use crate::{
    signer::GetPublicKey,
    types::{
        ics::core::ics24_host::path::DenomTrace,
        proto::cosmos::bank::v1beta1::{
            query_client::QueryClient as BankQueryClient, QueryBalanceRequest,
        },
    },
    ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
};

/// State of an IBC enabled chain
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainState {
    /// ID of chain
    pub id: ChainId,
    /// Node ID of chain
    pub node_id: NodeId,
    /// Configuration for chain
    pub config: ChainConfig,
    /// Consensus timestamp of solo machine (used when creating transactions on chain)
    pub consensus_timestamp: OffsetDateTime,
    /// Sequence of solo machine (used when creating transactions on chain)
    pub sequence: u32,
    /// Packet sequence of solo machine (used when creating transactions on chain)
    pub packet_sequence: u32,
    /// IBC connection details
    pub connection_details: Option<ConnectionDetails>,
    /// Creation time of chain
    pub created_at: OffsetDateTime,
    /// Last updation time of chain
    pub updated_at: OffsetDateTime,
}

/// IBC connection details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    /// Client ID of solo machine client on IBC enabled chain
    pub solo_machine_client_id: ClientId,
    /// Client ID of IBC enabled chain on solo machine
    pub tendermint_client_id: ClientId,
    /// Connection ID of solo machine client on IBC enabled chain
    pub solo_machine_connection_id: ConnectionId,
    /// Connection ID of IBC enabled chain on solo machine
    pub tendermint_connection_id: ConnectionId,
    /// Channel ID of solo machine client on IBC enabled chain
    pub solo_machine_channel_id: Option<ChannelId>,
    /// Channel ID of IBC enabled chain on solo machine
    pub tendermint_channel_id: Option<ChannelId>,
}

/// Configuration related to an IBC enabled chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfig {
    /// gRPC address
    pub grpc_addr: Url,
    /// RPC address
    pub rpc_addr: Url,
    /// Fee and gas limits
    pub fee: Fee,
    /// Trust level (e.g. 1/3)
    pub trust_level: Ratio<u64>,
    /// Trusting period
    pub trusting_period: Duration,
    /// Maximum clock drift
    pub max_clock_drift: Duration,
    /// RPC timeout duration
    pub rpc_timeout: Duration,
    /// Diversifier used in transactions for chain
    pub diversifier: String,
    /// Port ID used to create connection with chain
    pub port_id: PortId,
    /// Trusted height of the chain
    pub trusted_height: u32,
    /// Block hash at trusted height of the chain
    #[serde(with = "hex::serde")]
    pub trusted_hash: [u8; 32],
}

/// Fee and gas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    /// Fee amount
    pub amount: Decimal,
    /// Denom of fee
    pub denom: Identifier,
    /// Gas limit
    pub gas_limit: u64,
}

/// Signer's public key entry for an IBC enabled chain
#[derive(Debug, Serialize, Deserialize)]
pub struct ChainKey {
    /// ID of key
    pub id: i64,
    /// Chain ID
    pub chain_id: ChainId,
    /// Public key of signer
    pub public_key: String,
    /// Creation time of chain key entry
    pub created_at: OffsetDateTime,
}

/// Signer's public key entry for an IBC enabled chain
#[derive(Debug, Serialize)]
pub struct ChainKeyRequest<'a> {
    /// Chain ID
    pub chain_id: &'a ChainId,
    /// Public key of signer
    pub public_key: &'a str,
    /// Creation time of chain key entry
    pub created_at: OffsetDateTime,
}

impl ChainState {
    /// Returns the IBC denom of given denomination based on connection details. Returns `None` if connection details
    /// are not present.
    pub fn get_ibc_denom(&self, denom: &Identifier) -> Result<String> {
        let connection_details = self.connection_details.as_ref();
        ensure!(
            connection_details.is_some(),
            "connection is not established with given chain"
        );
        let connection_details = connection_details.unwrap();
        ensure!(
            connection_details.solo_machine_channel_id.is_some(),
            "can't find solo machine channel, channel is already closed"
        );

        let denom_trace = DenomTrace::new(
            &self.config.port_id,
            connection_details.solo_machine_channel_id.as_ref().unwrap(),
            denom,
        );

        let hash = Sha256::digest(denom_trace.to_string().as_bytes());

        Ok(format!("ibc/{}", hex::encode_upper(hash)))
    }

    /// Fetches on-chain balance of given denom
    pub async fn get_balance(
        &self,
        signer: impl GetPublicKey,
        denom: &Identifier,
    ) -> Result<Decimal> {
        let mut query_client = get_bank_query_client(self.config.grpc_addr.to_string()).await?;

        let denom = self.get_ibc_denom(denom)?;

        let request = QueryBalanceRequest {
            address: signer.to_account_address(&self.id)?,
            denom,
        };

        Ok(query_client
            .balance(request)
            .await?
            .into_inner()
            .balance
            .map(|coin| coin.amount.parse())
            .transpose()?
            .unwrap_or_default())
    }
}

#[cfg(feature = "wasm")]
async fn get_bank_query_client(grpc_addr: String) -> Result<BankQueryClient<Client>> {
    let grpc_client = Client::new(grpc_addr);
    Ok(BankQueryClient::new(grpc_client))
}

#[cfg(not(feature = "wasm"))]
async fn get_bank_query_client(grpc_addr: String) -> Result<BankQueryClient<Channel>> {
    BankQueryClient::connect(grpc_addr)
        .await
        .context("error when initializing grpc client")
}
