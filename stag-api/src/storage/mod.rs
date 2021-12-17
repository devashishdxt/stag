mod builder;

#[cfg(feature = "indexed-db-storage")]
pub use self::builder::IndexedDb;
pub use self::builder::StorageConfig;

#[cfg(feature = "indexed-db-storage")]
pub mod indexed_db_storage;

use anyhow::Result;
use async_trait::async_trait;
use tendermint::node::Id as NodeId;

use crate::{ChainConfig, ChainId, ChainKey, ChainState};

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Transaction: Storage {
    async fn done(self) -> Result<()>;
}

pub trait TransactionProvider: Storage {
    type Transaction: Transaction;

    fn transaction(&self, access_points: &[&str]) -> Result<Self::Transaction>;
}

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Storage {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()>;

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>>;

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()>;

    async fn get_chain_keys(&self, chain_id: &ChainId) -> Result<Vec<ChainKey>>;
}
