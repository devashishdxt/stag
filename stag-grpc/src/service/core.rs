tonic::include_proto!("core");

use std::time::Duration;

use k256::ecdsa::VerifyingKey;
use stag_api::{
    event::{EventHandlerConfig, TracingEventHandler},
    signer::Signer,
    stag::{Stag, StagBuilder},
    storage::TransactionProvider,
    tendermint::{JsonRpcConfig, ReqwestClient},
    types::{
        chain_state::{ChainConfig as StagChainConfig, Fee},
        ics::core::ics24_host::identifier::{ChainId, PortId},
        public_key::{PublicKey, PublicKeyAlgo},
    },
};
use tonic::{Request, Response, Status};

use self::core_server::Core;

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
const DEFAULT_PACKET_TIMEOUT_HEIGHT_OFFSET: u64 = 10;

const DEFAULT_MEMO: &str = "";

pub struct CoreService<S, T>
where
    S: Signer + Clone + 'static,
    T: TransactionProvider + 'static,
{
    stag: Stag<
        StagBuilder<
            S,
            T,
            <ReqwestClient as JsonRpcConfig>::Client,
            <TracingEventHandler as EventHandlerConfig>::EventHandler,
        >,
    >,
}

impl<S, T> CoreService<S, T>
where
    S: Signer + Clone + 'static,
    T: TransactionProvider + 'static,
{
    pub fn new(
        stag: Stag<
            StagBuilder<
                S,
                T,
                <ReqwestClient as JsonRpcConfig>::Client,
                <TracingEventHandler as EventHandlerConfig>::EventHandler,
            >,
        >,
    ) -> Self {
        Self { stag }
    }
}

#[tonic::async_trait]
impl<S, T> Core for CoreService<S, T>
where
    S: Signer + Clone + 'static,
    T: TransactionProvider + 'static,
{
    async fn add_chain(
        &self,
        request: Request<AddChainRequest>,
    ) -> Result<Response<AddChainResponse>, Status> {
        let request = request.into_inner();

        let config = request
            .config
            .ok_or_else(|| Status::invalid_argument("config must be provided"))?;

        let grpc_addr = config
            .grpc_addr
            .unwrap_or_else(|| DEFAULT_GRPC_ADDR.to_string())
            .parse()
            .map_err(|err: url::ParseError| Status::invalid_argument(err.to_string()))?;

        let rpc_addr = config
            .rpc_addr
            .unwrap_or_else(|| DEFAULT_RPC_ADDR.to_string())
            .parse()
            .map_err(|err: url::ParseError| Status::invalid_argument(err.to_string()))?;

        let fee_config = config.fee_config.unwrap_or_else(|| FeeConfig {
            fee_amount: Some(DEFAULT_FEE_AMOUNT.to_string()),
            fee_denom: Some(DEFAULT_FEE_DENOM.to_string()),
            gas_limit: Some(DEFAULT_GAS_LIMIT),
        });

        let fee = Fee {
            amount: fee_config
                .fee_amount
                .unwrap_or_else(|| DEFAULT_FEE_AMOUNT.to_string())
                .parse()
                .map_err(|err: rust_decimal::Error| Status::invalid_argument(err.to_string()))?,
            denom: fee_config
                .fee_denom
                .unwrap_or_else(|| DEFAULT_FEE_DENOM.to_string())
                .parse()
                .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?,
            gas_limit: fee_config.gas_limit.unwrap_or(DEFAULT_GAS_LIMIT),
        };

        let trust_level = config
            .trust_level
            .unwrap_or_else(|| DEFAULT_TRUST_LEVEL.to_string())
            .parse()
            .map_err(|e: num_rational::ParseRatioError| Status::invalid_argument(e.to_string()))?;

        let trusting_period = config
            .trusting_period
            .map(Duration::try_from)
            .transpose()
            .map_err(|_| Status::invalid_argument("negative trusting_period"))?
            .unwrap_or(DEFAULT_TRUSTING_PERIOD);

        let max_clock_drift = config
            .max_clock_drift
            .map(Duration::try_from)
            .transpose()
            .map_err(|_| Status::invalid_argument("negative max_clock_drift"))?
            .unwrap_or(DEFAULT_MAX_CLOCK_DRIFT);

        let rpc_timeout = config
            .rpc_timeout
            .map(Duration::try_from)
            .transpose()
            .map_err(|_| Status::invalid_argument("negative rpc_timeout"))?
            .unwrap_or(DEFAULT_RPC_TIMEOUT);

        let diversifier = config
            .diversifier
            .unwrap_or_else(|| DEFAULT_DIVERSIFIER.to_string());

        let trusted_height = config
            .trusted_height
            .ok_or_else(|| Status::invalid_argument("trusted_height must be provided"))?;

        let trusted_hash_str = config
            .trusted_hash
            .ok_or_else(|| Status::invalid_argument("trusted_hash must be provided"))?;

        if trusted_hash_str.is_empty() {
            return Err(Status::invalid_argument("trusted_hash is empty"));
        }

        let trusted_hash_bytes = hex::decode(&trusted_hash_str)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        if trusted_hash_bytes.len() != 32 {
            return Err(Status::invalid_argument("trusted_hash length should be 32"));
        }

        let mut trusted_hash = [0; 32];
        trusted_hash.copy_from_slice(&trusted_hash_bytes);

        let packet_timeout_height_offset = config
            .packet_timeout_height_offset
            .unwrap_or(DEFAULT_PACKET_TIMEOUT_HEIGHT_OFFSET);

        let chain_config = StagChainConfig {
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
        };

        let chain_id = self
            .stag
            .add_chain(&chain_config)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(AddChainResponse {
            chain_id: chain_id.to_string(),
        }))
    }

    async fn connect(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<ConnectResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;
        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());
        let request_id = request.request_id;
        let force = request.force;

        self.stag
            .connect(chain_id, request_id, memo, force)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(ConnectResponse {}))
    }

    async fn create_transfer_channel(
        &self,
        request: Request<CreateTransferChannelRequest>,
    ) -> Result<Response<CreateTransferChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;
        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());
        let request_id = request.request_id;

        self.stag
            .create_transfer_channel(chain_id, request_id, memo)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CreateTransferChannelResponse {}))
    }

    async fn close_transfer_channel(
        &self,
        request: Request<CloseTransferChannelRequest>,
    ) -> Result<Response<CloseTransferChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;
        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());
        let request_id = request.request_id;

        self.stag
            .close_channel(chain_id, &PortId::transfer(), request_id, memo)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CloseTransferChannelResponse {}))
    }

    async fn create_ica_channel(
        &self,
        request: Request<CreateIcaChannelRequest>,
    ) -> Result<Response<CreateIcaChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;
        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());
        let request_id = request.request_id;

        self.stag
            .create_ica_channel(chain_id, request_id, memo)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CreateIcaChannelResponse {}))
    }

    async fn close_ica_channel(
        &self,
        request: Request<CloseIcaChannelRequest>,
    ) -> Result<Response<CloseIcaChannelResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;
        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());
        let request_id = request.request_id;

        self.stag
            .close_channel(chain_id, &PortId::ica_controller(), request_id, memo)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CloseIcaChannelResponse {}))
    }

    async fn update_signer(
        &self,
        request: Request<UpdateSignerRequest>,
    ) -> Result<Response<UpdateSignerResponse>, Status> {
        let request = request.into_inner();

        let chain_id: ChainId = request
            .chain_id
            .parse()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?;

        let request_id = request.request_id;

        let memo = request.memo.unwrap_or_else(|| DEFAULT_MEMO.to_owned());

        let new_public_key_bytes = hex::decode(&request.new_public_key)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let new_verifying_key = VerifyingKey::from_sec1_bytes(&new_public_key_bytes)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let public_key_algo = request
            .public_key_algo
            .map(|s| s.parse())
            .transpose()
            .map_err(|err: anyhow::Error| Status::invalid_argument(err.to_string()))?
            .unwrap_or(PublicKeyAlgo::Secp256k1);

        let new_public_key = match public_key_algo {
            PublicKeyAlgo::Secp256k1 => PublicKey::Secp256k1(new_verifying_key),
            #[cfg(feature = "ethermint")]
            PublicKeyAlgo::EthSecp256k1 => PublicKey::EthSecp256k1(new_verifying_key),
        };

        self.stag
            .update_signer(chain_id, request_id, new_public_key, memo)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(UpdateSignerResponse {}))
    }
}
