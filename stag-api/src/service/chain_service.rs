use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use tendermint::node::Id as NodeId;

use crate::{
    event::{Event, EventHandler},
    signer::GetPublicKey,
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::tendermint_client::TendermintClient,
    ChainConfig, ChainId, ChainKey, ChainState, Identifier,
};

pub struct ChainService<S, E>
where
    S: TransactionProvider,
    E: EventHandler,
{
    storage: S,
    event_handler: Option<E>,
}

impl<S, E> ChainService<S, E>
where
    S: TransactionProvider,
    E: EventHandler,
{
    /// Creates a new instance of chain service
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            event_handler: None,
        }
    }

    /// Adds an event handler to chain service
    pub fn with_event_handler<NE>(self, event_handler: NE) -> ChainService<S, NE>
    where
        NE: EventHandler,
    {
        ChainService {
            storage: self.storage,
            event_handler: Some(event_handler),
        }
    }

    /// Add details of an IBC enabled chain
    pub async fn add<T>(&self, config: &ChainConfig, signer: &T) -> Result<ChainId>
    where
        T: GetPublicKey,
    {
        let tendermint_client = TendermintClient::new(config.rpc_addr.clone());
        let status = tendermint_client.status().await?;

        let chain_id: ChainId = status.node_info.network.to_string().parse()?;
        let node_id: NodeId = status.node_info.id;
        let public_key = signer.get_public_key(&chain_id)?.to_string();

        let transaction = self
            .storage
            .transaction(&["add_chain_state", "add_chain_key"])?;

        transaction
            .add_chain_state(chain_id.clone(), node_id, config.clone())
            .await?;
        transaction.add_chain_key(&chain_id, &public_key).await?;

        transaction
            .done()
            .await
            .map_err(|e| anyhow!("unable to commit transaction for adding IBC chain: {}", e))?;

        // TODO: send event via a channel to not block the current task (i.e. the caller)
        if let Some(ref event_handler) = self.event_handler {
            event_handler
                .handle(Event::ChainAdded {
                    chain_id: chain_id.clone(),
                })
                .await?;
        }

        Ok(chain_id)
    }

    /// Fetches details of a chain
    pub async fn get(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        self.storage.get_chain_state(chain_id).await
    }

    /// Returns the final denom of a token on solo machine after sending it on given chain
    pub async fn get_ibc_denom(&self, chain_id: &ChainId, denom: &Identifier) -> Result<String> {
        let chain = self
            .get(chain_id)
            .await?
            .ok_or_else(|| anyhow!("chain details not found when computing ibc denom"))?;
        chain.get_ibc_denom(denom)
    }

    /// Fetches all the public keys associated with solo machine client on given chain
    ///
    /// TODO: Add `limit` and `offset` parameters to fetch only a subset of keys
    pub async fn get_public_keys(&self, chain_id: &ChainId) -> Result<Vec<ChainKey>> {
        self.storage.get_chain_keys(chain_id).await
    }

    /// Fetches balance of given denom on IBC enabled chain
    pub async fn balance(
        &self,
        signer: impl GetPublicKey,
        chain_id: &ChainId,
        denom: &Identifier,
    ) -> Result<Decimal> {
        let chain_state = self
            .get(chain_id)
            .await?
            .ok_or_else(|| anyhow!("chain details not found when fetching balance"))?;

        chain_state.get_balance(signer, denom).await
    }
}
