use std::time::Duration;

use num_rational::Ratio;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tendermint::{block::Height as BlockHeight, node::Id as NodeId};
use time::OffsetDateTime;
use url::Url;

use crate::{ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId};

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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    /// Fee amount
    pub amount: Decimal,
    /// Denom of fee
    pub denom: Identifier,
    /// Gas limit
    pub gas_limit: u64,
}
