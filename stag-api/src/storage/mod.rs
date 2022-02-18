//! Data storage backends used by Stag API
mod builder;
#[cfg(feature = "indexed-db-storage")]
mod indexed_db_storage;
#[cfg(feature = "sqlite-storage")]
mod sql_db_storage;
mod storage_traits;

#[cfg(feature = "indexed-db-storage")]
pub use self::builder::IndexedDb;
#[cfg(feature = "sqlite-storage")]
pub use self::builder::Sqlite;
pub use self::{
    builder::StorageConfig,
    storage_traits::{Storage, Transaction, TransactionProvider},
};

#[derive(Clone)]
/// A no-op storage backend
pub struct NoopStorage;
