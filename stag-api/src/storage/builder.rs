use anyhow::Result;
use async_trait::async_trait;
use sealed::sealed;

use crate::trait_util::Base;

#[cfg(feature = "indexed-db-storage")]
use super::indexed_db_storage::IndexedDbStorage;
#[cfg(feature = "sqlite-storage")]
use super::sql_db_storage::SqlDbStorage;
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

#[cfg_attr(feature = "doc", doc(cfg(feature = "sqlite-storage")))]
#[cfg(feature = "sqlite-storage")]
#[derive(Debug, Clone, PartialEq, Eq)]
/// Storage backend using SQLite database
pub struct Sqlite {
    uri: String,
}

#[cfg(feature = "sqlite-storage")]
impl Sqlite {
    /// Creates a new instance of Sqlite
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
        }
    }

    async fn get_sqlite_storage(self) -> Result<SqlDbStorage> {
        SqlDbStorage::new(self.uri).await
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
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
#[async_trait(?Send)]
#[sealed]
impl StorageConfig for IndexedDb {
    type Storage = IndexedDbStorage;

    async fn into_storage(self) -> Result<Self::Storage> {
        self.get_indexed_db_storage().await
    }
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "sqlite-storage")))]
#[cfg(feature = "sqlite-storage")]
#[async_trait]
#[sealed]
impl StorageConfig for Sqlite {
    type Storage = SqlDbStorage;

    async fn into_storage(self) -> Result<Self::Storage> {
        self.get_sqlite_storage().await
    }
}

#[cfg(all(test, feature = "non-wasm"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_storage() {
        let config = Sqlite::new("sqlite::memory:");
        assert!(config.into_storage().await.is_ok());
    }
}
