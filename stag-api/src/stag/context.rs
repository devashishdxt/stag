//! Context for the Stag API
use anyhow::Result;

use crate::{event::EventHandler, storage::TransactionProvider, trait_util::Base};

/// Context for the Stag API
pub trait StagContext: Base {
    /// Type of signer used by the Stag API
    type Signer;
    /// Type of storage used by the Stag API
    type Storage;
    /// Type of rpc client used by the Stag API
    type RpcClient;
    /// Type of event handler used by the Stag API
    type EventHandler: EventHandler;

    /// Returns the signer used by the Stag API
    fn signer(&self) -> &Self::Signer;

    /// Returns the storage used by the Stag API
    fn storage(&self) -> &Self::Storage;

    /// Returns the rpc client used by the Stag API
    fn rpc_client(&self) -> &Self::RpcClient;

    /// Returns the event handler used by the Stag API
    fn event_handler(&self) -> Option<&Self::EventHandler>;

    /// Returns all the individual components of stag context
    fn unwrap(
        self,
    ) -> (
        Self::Signer,
        Self::Storage,
        Self::RpcClient,
        Option<Self::EventHandler>,
    );
}

/// Obtain a context with database transaction
pub trait WithTransaction: StagContext
where
    Self::Storage: TransactionProvider,
{
    /// Type of context
    type TransactionContext: StagContext<
        Signer = Self::Signer,
        Storage = <Self::Storage as TransactionProvider>::Transaction,
        RpcClient = Self::RpcClient,
        EventHandler = Self::EventHandler,
    >;

    /// Create a context with database transaction
    fn with_transaction(&self) -> Result<Self::TransactionContext>;
}
