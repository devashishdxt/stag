//! Data storage backends used by Stag API
mod builder;
#[cfg(feature = "indexed-db-storage")]
mod indexed_db_storage;
mod storage_traits;

#[cfg(feature = "indexed-db-storage")]
pub use self::builder::IndexedDb;
pub use self::{
    builder::StorageConfig,
    storage_traits::{Storage, Transaction, TransactionProvider},
};

/// A no-op storage backend
pub struct NoopStorage;
