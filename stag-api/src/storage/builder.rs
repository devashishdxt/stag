use anyhow::Result;
use async_trait::async_trait;
use sealed::sealed;

#[cfg(feature = "indexed-db-storage")]
use super::indexed_db_storage::IndexedDbStorage;
use super::TransactionProvider;

#[cfg(feature = "indexed-db-storage")]
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
pub trait StorageConfig {
    type Storage: TransactionProvider;

    async fn into_storage(self) -> Result<Self::Storage>;
}

#[cfg(feature = "indexed-db-storage")]
#[async_trait(?Send)]
#[sealed]
impl StorageConfig for IndexedDb {
    type Storage = IndexedDbStorage;

    async fn into_storage(self) -> Result<Self::Storage> {
        self.get_indexed_db_storage().await
    }
}
