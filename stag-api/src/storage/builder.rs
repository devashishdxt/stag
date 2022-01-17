use anyhow::Result;
use async_trait::async_trait;
use sealed::sealed;

use crate::trait_util::Base;

#[cfg(feature = "indexed-db-storage")]
use super::indexed_db_storage::IndexedDbStorage;
use super::TransactionProvider;

#[cfg_attr(feature = "doc", doc(cfg(feature = "indexed-db-storage")))]
#[cfg(feature = "indexed-db-storage")]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Storage backend for browsers using Indexed DB
pub struct IndexedDb {
    name: String,
}

#[cfg(feature = "indexed-db-storage")]
impl IndexedDb {
    /// Creates a new instance of IndexedDb
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    async fn get_indexed_db_storage(self) -> Result<IndexedDbStorage> {
        IndexedDbStorage::new(&self.name).await
    }
}

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
#[sealed]
/// Configuration for storage backend
pub trait StorageConfig: Base {
    /// Concrete storage backend type that this config will produce
    type Storage: TransactionProvider;

    /// Create concrete storage backend from this config
    async fn into_storage(self) -> Result<Self::Storage>;
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "indexed-db-storage")))]
#[cfg(feature = "indexed-db-storage")]
#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
#[sealed]
impl StorageConfig for IndexedDb {
    type Storage = IndexedDbStorage;

    async fn into_storage(self) -> Result<Self::Storage> {
        self.get_indexed_db_storage().await
    }
}
