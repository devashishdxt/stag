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

use crate::proto::transfer::{
    transfer_server::Transfer, BurnRequest, BurnResponse, MintRequest, MintResponse,
};

pub struct TransferService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> TransferService<C>
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
impl<C> Transfer for TransferService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    async fn mint(&self, request: Request<MintRequest>) -> Result<Response<MintResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let amount = U256::from_dec_str(&request.amount)
            .context("invalid amount")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let denom = request
            .denom
            .parse()
            .context("invalid denom")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let receiver = request.receiver_address;

        let memo = request.memo.unwrap_or_default();

        let transaction_hash = self
            .stag
            .read()
            .await
            .mint(chain_id, request_id, amount, denom, receiver, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(MintResponse { transaction_hash }))
    }

    async fn burn(&self, request: Request<BurnRequest>) -> Result<Response<BurnResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

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
            .burn(chain_id, request_id, amount, denom, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(BurnResponse { transaction_hash }))
    }
}
