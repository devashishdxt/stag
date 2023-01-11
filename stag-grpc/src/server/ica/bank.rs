use std::sync::Arc;

use anyhow::{Context, Error};
use primitive_types::U256;
use stag_api::{
    signer::Signer,
    stag::{Stag, StagContext, WithTransaction},
    storage::TransactionProvider,
    tendermint::JsonRpcClient,
    types::ics::core::ics24_host::identifier::PortId,
};
use tokio::sync::RwLock;
use tonic::{async_trait, Request, Response, Status};

use crate::proto::ica::bank::{ica_bank_server::IcaBank, SendRequest, SendResponse};

pub struct IcaBankService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> IcaBankService<C>
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
impl<C> IcaBank for IcaBankService<C>
where
    C: StagContext + WithTransaction + 'static,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    async fn send(&self, request: Request<SendRequest>) -> Result<Response<SendResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let request_id = request.request_id;

        let to_address = request.to_address;

        let amount = U256::from_dec_str(&request.amount)
            .context("invalid amount")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let mut denom = request
            .denom
            .parse()
            .context("invalid denom")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        if request.ibc_denom {
            denom = self
                .stag
                .read()
                .await
                .get_ibc_denom(&chain_id, &PortId::transfer(), &denom)
                .await
                .map_err(|err| Status::internal(format!("{err:?}")))?
                .parse()
                .context("unable to parse ibc denom")
                .map_err(|err: Error| Status::invalid_argument(format!("{err:?}")))?;
        }

        let memo = request.memo.unwrap_or_default();

        let transaction_hash = self
            .stag
            .read()
            .await
            .ica_send(chain_id, request_id, to_address, amount, denom, memo)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(SendResponse { transaction_hash }))
    }
}
