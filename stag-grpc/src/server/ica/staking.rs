use std::sync::Arc;

use anyhow::Context;
use primitive_types::U256;
use stag_api::{
    signer::Signer,
    stag::{Stag, StagContext, WithTransaction},
    storage::TransactionProvider,
    tendermint::JsonRpcClient,
};
use tokio::sync::RwLock;
use tonic::{async_trait, Request, Response, Status};

use crate::proto::ica::staking::{
    ica_staking_server::IcaStaking, DelegateRequest, DelegateResponse, UndelegateRequest,
    UndelegateResponse,
};

pub struct IcaStakingService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> IcaStakingService<C>
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
impl<C> IcaStaking for IcaStakingService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    async fn delegate(
        &self,
        request: Request<DelegateRequest>,
    ) -> Result<Response<DelegateResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let validator_address = request.validator_address;

        let amount = U256::from_dec_str(&request.amount)
            .context("invalid amount")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let denom = request
            .denom
            .parse()
            .context("invalid denom")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let memo = request.memo.unwrap_or_default();

        let transaction_hash = self
            .stag
            .read()
            .await
            .ica_delegate(chain_id, request_id, validator_address, amount, denom, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(DelegateResponse { transaction_hash }))
    }

    async fn undelegate(
        &self,
        request: Request<UndelegateRequest>,
    ) -> Result<Response<UndelegateResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let validator_address = request.validator_address;

        let amount = U256::from_dec_str(&request.amount)
            .context("invalid amount")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let denom = request
            .denom
            .parse()
            .context("invalid denom")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let memo = request.memo.unwrap_or_default();

        let transaction_hash = self
            .stag
            .read()
            .await
            .ica_undelegate(chain_id, request_id, validator_address, amount, denom, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(UndelegateResponse { transaction_hash }))
    }
}
