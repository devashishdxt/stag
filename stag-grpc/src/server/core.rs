use std::{sync::Arc, time::Duration};

use anyhow::{ensure, Context, Error, Result};
use stag_api::{
    signer::Signer,
    stag::{Stag, StagContext, WithTransaction},
    storage::TransactionProvider,
    tendermint::JsonRpcClient,
    types::{
        chain_state::{ChainConfig, Fee},
        ics::core::ics24_host::identifier::PortId,
        public_key::{PublicKey, PublicKeyAlgo},
    },
};
use tokio::sync::RwLock;
use tonic::{async_trait, Request, Response, Status};

use crate::proto::core::{
    core_server::Core, AddChainRequest, AddChainResponse, CloseChannelRequest,
    CloseChannelResponse, ConnectChainRequest, ConnectChainResponse, CreateChannelRequest,
    CreateChannelResponse, FeeConfig, UpdateSignerRequest, UpdateSignerResponse,
};

const DEFAULT_GRPC_ADDR: &str = "http://0.0.0.0:9090";
const DEFAULT_RPC_ADDR: &str = "http://0.0.0.0:26657";
const DEFAULT_FEE_AMOUNT: &str = "1000";
const DEFAULT_FEE_DENOM: &str = "stake";
const DEFAULT_GAS_LIMIT: u64 = 300000;
const DEFAULT_TRUST_LEVEL: &str = "1/3";
const DEFAULT_TRUSTING_PERIOD: Duration = Duration::from_secs(336 * 60 * 60); // 14 days
const DEFAULT_MAX_CLOCK_DRIFT: Duration = Duration::from_secs(3); // 3 secs
const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(60); // 60 secs
const DEFAULT_DIVERSIFIER: &str = "stag";
const DEFAULT_PACKET_TIMEOUT_HEIGHT_OFFSET: u64 = 20;

pub struct CoreService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> CoreService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    pub fn new(stag: Arc<RwLock<Stag<C>>>) -> Self {
        Self { stag }
    }
}

#[async_trait]
impl<C> Core for CoreService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    async fn add_chain(
        &self,
        request: Request<AddChainRequest>,
    ) -> Result<Response<AddChainResponse>, Status> {
        let chain_config: ChainConfig = request
            .into_inner()
            .try_into()
            .map_err(|err: Error| Status::invalid_argument(format!("{err:?}")))?;

        let chain_id = self
            .stag
            .read()
            .await
            .add_chain(&chain_config)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(AddChainResponse {
            chain_id: chain_id.to_string(),
        }))
    }

    async fn connect_chain(
        &self,
        request: Request<ConnectChainRequest>,
    ) -> Result<Response<ConnectChainResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_default();

        let force = request.force;

        self.stag
            .read()
            .await
            .connect(chain_id, request_id, memo, force)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(ConnectChainResponse {}))
    }

    async fn create_transfer_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<CreateChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_default();

        self.stag
            .read()
            .await
            .create_transfer_channel(chain_id, request_id, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(CreateChannelResponse {}))
    }

    async fn create_ica_channel(
        &self,
        request: Request<CreateChannelRequest>,
    ) -> Result<Response<CreateChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_default();

        self.stag
            .read()
            .await
            .create_ica_channel(chain_id, request_id, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(CreateChannelResponse {}))
    }

    async fn close_transfer_channel(
        &self,
        request: Request<CloseChannelRequest>,
    ) -> Result<Response<CloseChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_default();

        self.stag
            .read()
            .await
            .close_channel(chain_id, &PortId::transfer(), request_id, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(CloseChannelResponse {}))
    }

    async fn close_ica_channel(
        &self,
        request: Request<CloseChannelRequest>,
    ) -> Result<Response<CloseChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_default();

        self.stag
            .read()
            .await
            .close_channel(chain_id, &PortId::ica_controller(), request_id, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(CloseChannelResponse {}))
    }

    async fn update_signer(
        &self,
        request: Request<UpdateSignerRequest>,
    ) -> Result<Response<UpdateSignerResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let new_public_key_algo = request
            .new_public_key_algo
            .map(|algo| algo.parse())
            .transpose()
            .context("invalid algo")
            .map_err(|err: Error| Status::invalid_argument(format!("{err:?}")))?
            .unwrap_or(PublicKeyAlgo::Secp256k1);

        let new_public_key = PublicKey::new(request.new_public_key, new_public_key_algo)
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let memo = request.memo.unwrap_or_default();

        self.stag
            .read()
            .await
            .update_signer(chain_id, request_id, new_public_key, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(UpdateSignerResponse {}))
    }
}

impl TryFrom<AddChainRequest> for ChainConfig {
    type Error = Error;

    fn try_from(value: AddChainRequest) -> Result<Self, Self::Error> {
        let grpc_addr = value
            .grpc_addr
            .unwrap_or_else(|| DEFAULT_GRPC_ADDR.to_string())
            .parse()
            .context("invalid gRPC address")?;

        let rpc_addr = value
            .rpc_addr
            .unwrap_or_else(|| DEFAULT_RPC_ADDR.to_string())
            .parse()
            .context("invalid RPC address")?;

        let fee = value
            .fee_config
            .map(|fee_config: FeeConfig| -> Result<Fee> {
                Ok(Fee {
                    amount: fee_config
                        .fee_amount
                        .unwrap_or_else(|| DEFAULT_FEE_AMOUNT.to_string())
                        .parse()
                        .context("invalid fee amount")?,
                    denom: fee_config
                        .fee_denom
                        .unwrap_or_else(|| DEFAULT_FEE_DENOM.to_string())
                        .parse()
                        .context("invalid fee denom")?,
                    gas_limit: fee_config.gas_limit.unwrap_or(DEFAULT_GAS_LIMIT),
                })
            })
            .unwrap_or_else(|| {
                Ok(Fee {
                    amount: DEFAULT_FEE_AMOUNT.parse().context("invalid fee amount")?,
                    denom: DEFAULT_FEE_DENOM.parse().context("invalid fee denom")?,
                    gas_limit: DEFAULT_GAS_LIMIT,
                })
            })?;

        let trust_level = value
            .trust_level
            .unwrap_or_else(|| DEFAULT_TRUST_LEVEL.to_string())
            .parse()
            .context("context trust level")?;

        let trusting_period = value
            .trusting_period
            .map(|trusting_period| {
                Duration::try_from(trusting_period).context("context trusting period")
            })
            .transpose()?
            .unwrap_or(DEFAULT_TRUSTING_PERIOD);

        let max_clock_drift = value
            .max_clock_drift
            .map(|max_clock_drift| {
                Duration::try_from(max_clock_drift).context("context trusting period")
            })
            .transpose()?
            .unwrap_or(DEFAULT_MAX_CLOCK_DRIFT);

        let rpc_timeout = value
            .rpc_timeout
            .map(|rpc_timeout| Duration::try_from(rpc_timeout).context("context trusting period"))
            .transpose()?
            .unwrap_or(DEFAULT_RPC_TIMEOUT);

        let diversifier = value
            .diversifier
            .unwrap_or_else(|| DEFAULT_DIVERSIFIER.to_string());

        let trusted_height = value.trusted_height;

        let trusted_hash = parse_trusted_hash(&value.trusted_hash)?;

        let packet_timeout_height_offset = value
            .packet_timeout_height_offset
            .unwrap_or(DEFAULT_PACKET_TIMEOUT_HEIGHT_OFFSET);

        Ok(Self {
            grpc_addr,
            rpc_addr,
            fee,
            trust_level,
            trusting_period,
            max_clock_drift,
            rpc_timeout,
            diversifier,
            trusted_height,
            trusted_hash,
            packet_timeout_height_offset,
        })
    }
}

fn parse_trusted_hash(hash: &str) -> Result<[u8; 32]> {
    ensure!(!hash.is_empty(), "empty trusted hash");

    let bytes = hex::decode(hash).context("invalid trusted hash hex bytes")?;
    ensure!(bytes.len() == 32, "trusted hash length should be 32");

    let mut trusted_hash = [0; 32];
    trusted_hash.clone_from_slice(&bytes);

    Ok(trusted_hash)
}
