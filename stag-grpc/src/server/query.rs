use std::{str::FromStr, sync::Arc};

use anyhow::{Context, Error};
use prost_types::Timestamp;
use stag_api::{
    signer::Signer,
    stag::{Stag, StagContext},
    storage::Storage,
    types::{
        ics::core::ics24_host::identifier::PortId,
        operation::{Operation, OperationType},
    },
};
use tokio::sync::RwLock;
use tonic::{async_trait, Request, Response, Status};

use crate::proto::query::{
    op::OpType, query_server::Query, BurnOperation, GetBalanceRequest, GetBalanceResponse,
    GetHistoryRequest, GetHistoryResponse, GetIbcDenomRequest, GetIbcDenomResponse,
    IcaDelegateOperation, IcaSendOperation, IcaUndelegateOperation, MintOperation, Op,
};

pub struct QueryService<C>
where
    C: StagContext + 'static,
    C::Signer: Signer,
    C::Storage: Storage,
{
    stag: Arc<RwLock<Stag<C>>>,
}

impl<C> QueryService<C>
where
    C: StagContext + 'static,
    C::Signer: Signer,
    C::Storage: Storage,
{
    pub fn new(stag: Arc<RwLock<Stag<C>>>) -> Self {
        Self { stag }
    }
}

#[async_trait]
impl<C> Query for QueryService<C>
where
    C: StagContext + 'static,
    C::Signer: Signer,
    C::Storage: Storage,
{
    async fn get_balance(
        &self,
        request: Request<GetBalanceRequest>,
    ) -> Result<Response<GetBalanceResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
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

        let balance = self
            .stag
            .read()
            .await
            .get_balance(&chain_id, &denom)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(GetBalanceResponse {
            balance: balance.to_string(),
        }))
    }

    async fn get_history(
        &self,
        request: Request<GetHistoryRequest>,
    ) -> Result<Response<GetHistoryResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let limit = request.limit;

        let offset = request.offset;

        let operations = self
            .stag
            .read()
            .await
            .get_history(&chain_id, limit, offset)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(operations.try_into().map_err(
            |err: Error| Status::internal(format!("{err:?}")),
        )?))
    }

    async fn get_ibc_denom(
        &self,
        request: Request<GetIbcDenomRequest>,
    ) -> Result<Response<GetIbcDenomResponse>, Status> {
        let request = request.into_inner();

        let chain_id = request
            .chain_id
            .parse()
            .context("invalid chain id")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let denom = request
            .denom
            .parse()
            .context("invalid denom")
            .map_err(|err| Status::invalid_argument(format!("{err:?}")))?;

        let ibc_denom = self
            .stag
            .read()
            .await
            .get_ibc_denom(&chain_id, &PortId::transfer(), &denom)
            .await
            .map_err(|err| Status::internal(format!("{err:?}")))?;

        Ok(Response::new(GetIbcDenomResponse { ibc_denom }))
    }
}

impl TryFrom<Vec<Operation>> for GetHistoryResponse {
    type Error = Error;

    fn try_from(operations: Vec<Operation>) -> Result<Self, Self::Error> {
        let mut ops = Vec::with_capacity(operations.len());

        for operation in operations {
            let op = Op {
                id: operation.id,
                request_id: operation.request_id,
                chain_id: operation.chain_id.to_string(),
                port_id: operation.port_id.to_string(),
                transaction_hash: operation.transaction_hash,
                op_type: Some(operation.operation_type.into()),
                created_at: Some(
                    Timestamp::from_str(&operation.created_at.to_rfc3339())
                        .context("unable to parse protobuf timestamp")?,
                ),
            };

            ops.push(op);
        }

        Ok(GetHistoryResponse { operations: ops })
    }
}

impl From<OperationType> for OpType {
    fn from(operation_type: OperationType) -> Self {
        match operation_type {
            OperationType::Mint { to, denom, amount } => OpType::Mint(MintOperation {
                to,
                denom: denom.to_string(),
                amount: amount.to_string(),
            }),
            OperationType::Burn {
                from,
                denom,
                amount,
            } => OpType::Burn(BurnOperation {
                from,
                denom: denom.to_string(),
                amount: amount.to_string(),
            }),
            OperationType::IcaSend { to, denom, amount } => OpType::IcaSend(IcaSendOperation {
                to,
                denom: denom.to_string(),
                amount: amount.to_string(),
            }),
            OperationType::IcaDelegate {
                validator_address,
                denom,
                amount,
            } => OpType::IcaDelegate(IcaDelegateOperation {
                validator_address,
                denom: denom.to_string(),
                amount: amount.to_string(),
            }),
            OperationType::IcaUndelegate {
                validator_address,
                denom,
                amount,
            } => OpType::IcaUndelegate(IcaUndelegateOperation {
                validator_address,
                denom: denom.to_string(),
                amount: amount.to_string(),
            }),
        }
    }
}
