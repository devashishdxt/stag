use std::collections::HashSet;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rexie::{
    Index, KeyRange, ObjectStore, Rexie, Transaction as RexieTransaction, TransactionMode,
};
use tendermint::node::Id as NodeId;

use crate::{
    time_util::now_utc, types::chain_state::ChainKeyRequest, ChainConfig, ChainId, ChainKey,
    ChainState,
};

use super::{Storage, Transaction, TransactionProvider};

pub const CHAIN_STATE_STORE_NAME: &str = "chain_state";
pub const CHAIN_KEY_STORE_NAME: &str = "chain_key";

pub struct IndexedDbStorage {
    rexie: Rexie,
}

impl IndexedDbStorage {
    pub async fn new(name: &str) -> Result<Self> {
        let rexie = Rexie::builder(name)
            .version(1)
            .add_object_store(ObjectStore::new(CHAIN_STATE_STORE_NAME).key_path("id"))
            .add_object_store(
                ObjectStore::new(CHAIN_KEY_STORE_NAME)
                    .key_path("id")
                    .auto_increment(true)
                    .add_index(Index::new("chain_id", "chain_id")),
            )
            .build()
            .await
            .map_err(|err| anyhow!("error when opening indexed db: {}", err))?;

        Ok(Self { rexie })
    }
}

pub struct IndexedDbTransaction {
    transaction: RexieTransaction,
}

#[async_trait(?Send)]
impl Transaction for IndexedDbTransaction {
    async fn done(self) -> Result<()> {
        self.transaction
            .finish()
            .await
            .map_err(|err| anyhow!("error when committing indexed db transaction: {}", err))
    }
}

#[async_trait(?Send)]
impl Storage for IndexedDbTransaction {
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
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
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
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        let value = store
            .get(
                serde_wasm_bindgen::to_value(chain_id)
                    .map_err(|err| anyhow!("error when serializing chain_state: {}", err))?,
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

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        let store = self
            .transaction
            .store(CHAIN_KEY_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_key object store: {}", err))?;

        let chain_key = ChainKeyRequest {
            chain_id,
            public_key,
            created_at: now_utc()?,
        };

        store
            .put_value(
                serde_wasm_bindgen::to_value(&chain_key)
                    .map_err(|err| anyhow!("error when serializing chain_key: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in chain_key object store: {}",
                    err
                )
            })
    }

    async fn get_chain_keys(&self, chain_id: &ChainId) -> Result<Vec<ChainKey>> {
        let store = self
            .transaction
            .store(CHAIN_KEY_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_key object store: {}", err))?;

        let js_chain_id = serde_wasm_bindgen::to_value(&chain_id)
            .map_err(|err| anyhow!("error when serializing chain_id: {}", err))?;

        let chain_keys = store
            .get_all_key_range(
                KeyRange::only(&js_chain_id)
                    .map_err(|err| anyhow!("unable to generate keyrange: {}", err))?,
                None,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting all chain keys for chain id [{}]: {}",
                    chain_id,
                    err
                )
            })?;

        chain_keys
            .into_iter()
            .map(|value| {
                serde_wasm_bindgen::from_value(value)
                    .map_err(|err| anyhow!("error when deserializing chain_key: {}", err))
            })
            .collect()
    }
}

impl TransactionProvider for IndexedDbStorage {
    type Transaction = IndexedDbTransaction;

    fn transaction(&self, access_points: &[&str]) -> Result<Self::Transaction> {
        let mut store_names = HashSet::new();
        let mut is_write = false;

        for access_point in access_points {
            let (store_name, write_required) = match *access_point {
                "add_chain_state" => (CHAIN_STATE_STORE_NAME, true),
                "get_chain_state" => (CHAIN_STATE_STORE_NAME, false),
                "add_chain_key" => (CHAIN_KEY_STORE_NAME, true),
                "get_chain_keys" => (CHAIN_KEY_STORE_NAME, false),
                _ => return Err(anyhow!("unknown access point: {}", access_point)),
            };

            if write_required {
                is_write = true;
            }

            store_names.insert(store_name.to_string());
        }

        let mode = if is_write {
            TransactionMode::ReadWrite
        } else {
            TransactionMode::ReadOnly
        };

        let rexie_transaction = self
            .rexie
            .transaction(store_names.into_iter().collect(), mode)
            .map_err(|err| anyhow!("error when opening indexed db transaction: {}", err))?;

        Ok(IndexedDbTransaction {
            transaction: rexie_transaction,
        })
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
        let transaction = self.transaction(&["add_chain_state"])?;

        transaction
            .add_chain_state(chain_id, node_id, chain_config)
            .await?;

        transaction.done().await
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let transaction = self.transaction(&["get_chain_state"])?;

        let result = transaction.get_chain_state(chain_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        let transaction = self.transaction(&["add_chain_key"])?;

        transaction.add_chain_key(chain_id, public_key).await?;

        transaction.done().await?;

        Ok(())
    }

    async fn get_chain_keys(&self, chain_id: &ChainId) -> Result<Vec<ChainKey>> {
        let transaction = self.transaction(&["get_chain_keys"])?;

        let result = transaction.get_chain_keys(chain_id).await?;

        transaction.done().await?;

        Ok(result)
    }
}
