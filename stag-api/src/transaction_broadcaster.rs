use anyhow::Result;
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use tendermint::abci::{transaction::Hash, DeliverTx};

/// Responsible for broadcasting transactions to tendermint
#[derive(Debug)]
pub struct TransactionBroadcaster {
    rpc_url: String,
}

impl TransactionBroadcaster {
    /// Creates a new transaction broadcaster
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }

    /// Broadcasts transaction and returns transaction hash and deliver_tx response
    pub async fn broadcast(&self, transaction: TxRaw) -> Result<(Hash, DeliverTx)> {
        todo!()
    }
}
