use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rexie::{ObjectStore, Rexie, Transaction, TransactionMode};
use tendermint::node::Id as NodeId;

use crate::{time_util::now_utc, ChainConfig, ChainId, ChainState};

use super::Storage;

pub struct IndexedDbStorage {
    rexie: Rexie,
}

impl IndexedDbStorage {
    pub async fn new() -> Result<Self> {
        let rexie = Rexie::builder("solo-machine")
            .version(1)
            .add_object_store(ObjectStore::new("chain_state").key_path("id"))
            .build()
            .await
            .map_err(|err| anyhow!("error when opening indexed db: {}", err))?;

        Ok(Self { rexie })
    }

    pub fn transaction(
        &self,
        store_names: Vec<String>,
        mode: TransactionMode,
    ) -> Result<Transaction> {
        self.rexie
            .transaction(store_names, mode)
            .map_err(|err| anyhow!("error when opening indexed db transaction: {}", err))
    }
}

#[async_trait(?Send)]
impl Storage for Transaction {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let current_time = now_utc()?;

        let chain_state = ChainState {
            id: chain_id,
            node_id,
            config: chain_config,
            consensus_timestamp: current_time,
            sequence: 1,
            packet_sequence: 1,
            connection_details: None,
            created_at: current_time,
            updated_at: current_time,
        };

        let store = self
            .store("chain_state")
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        store
            .put_value(
                serde_wasm_bindgen::to_value(&chain_state)
                    .map_err(|err| anyhow!("error when serializing chain_state: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in chain_state object store: {}",
                    err
                )
            })
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let store = self
            .store("chain_state")
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        let value = store
            .get(
                serde_wasm_bindgen::to_value(chain_id)
                    .map_err(|err| anyhow!("error when serializing chain_id: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting value from chain_state object store: {}",
                    err
                )
            })?;

        serde_wasm_bindgen::from_value(value)
            .map_err(|err| anyhow!("error when deserializing chain_state: {}", err))
    }
}

#[async_trait(?Send)]
impl Storage for IndexedDbStorage {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let transaction =
            self.transaction(vec!["chain_state".to_string()], TransactionMode::ReadWrite)?;

        transaction
            .add_chain_state(chain_id, node_id, chain_config)
            .await?;

        transaction
            .finish()
            .await
            .map_err(|err| anyhow!("error when finishing indexed db transaction: {}", err))
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let transaction =
            self.transaction(vec!["chain_state".to_string()], TransactionMode::ReadOnly)?;

        let result = transaction.get_chain_state(chain_id).await?;

        transaction
            .finish()
            .await
            .map_err(|err| anyhow!("error when finishing indexed db transaction: {}", err))?;

        Ok(result)
    }
}
