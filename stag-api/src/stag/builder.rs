use anyhow::Result;
use async_trait::async_trait;

use crate::{
    event::{EventHandler, EventHandlerConfig, NoopEventHandler},
    signer::{NoopSigner, SignerConfig},
    storage::{NoopStorage, StorageConfig, TransactionProvider},
    tendermint::{JsonRpcConfig, NoopRpcClient},
    trait_util::Base,
};

use super::{Stag, StagContext, WithTransaction};

/// Builder for Stag API
pub struct StagBuilder<S, T, C, E> {
    signer: S,
    storage: T,
    rpc_client: C,
    event_handler: Option<E>,
}

impl Default for StagBuilder<NoopSigner, NoopStorage, NoopRpcClient, NoopEventHandler> {
    fn default() -> Self {
        Self {
            signer: NoopSigner,
            storage: NoopStorage,
            rpc_client: NoopRpcClient,
            event_handler: None,
        }
    }
}

impl StagBuilder<NoopSigner, NoopStorage, NoopRpcClient, NoopEventHandler> {
    /// Creates a new instance of stag builder
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S, T, C, E> StagBuilder<S, T, C, E>
where
    S: Base,
    T: Base,
    C: Base,
    E: EventHandler,
{
    /// Adds signer to the context
    pub fn with_signer<NS>(self, signer: NS) -> Result<StagBuilder<NS::Signer, T, C, E>>
    where
        NS: SignerConfig,
    {
        Ok(StagBuilder {
            signer: signer.into_signer()?,
            storage: self.storage,
            rpc_client: self.rpc_client,
            event_handler: self.event_handler,
        })
    }

    /// Adds storage to the context
    pub async fn with_storage<NT>(self, storage: NT) -> Result<StagBuilder<S, NT::Storage, C, E>>
    where
        NT: StorageConfig,
    {
        Ok(StagBuilder {
            signer: self.signer,
            storage: storage.into_storage().await?,
            rpc_client: self.rpc_client,
            event_handler: self.event_handler,
        })
    }

    /// Adds rpc client to the context
    pub fn with_rpc_client<NC>(self, rpc_client: NC) -> StagBuilder<S, T, NC::Client, E>
    where
        NC: JsonRpcConfig,
    {
        StagBuilder {
            signer: self.signer,
            storage: self.storage,
            rpc_client: rpc_client.into_client(),
            event_handler: self.event_handler,
        }
    }

    /// Adds event handler to the context
    pub fn with_event_handler<NE>(self, event_handler: NE) -> StagBuilder<S, T, C, NE::EventHandler>
    where
        NE: EventHandlerConfig,
    {
        StagBuilder {
            signer: self.signer,
            storage: self.storage,
            rpc_client: self.rpc_client,
            event_handler: Some(event_handler.into_event_handler()),
        }
    }

    /// Builds the Stag API
    pub fn build(self) -> Stag<Self> {
        self.into()
    }
}

impl<S, T, C, E> StagContext for StagBuilder<S, T, C, E>
where
    S: Base,
    T: Base,
    C: Base,
    E: EventHandler,
{
    type Signer = S;
    type Storage = T;
    type RpcClient = C;
    type EventHandler = E;

    fn signer(&self) -> &Self::Signer {
        &self.signer
    }

    fn storage(&self) -> &Self::Storage {
        &self.storage
    }

    fn rpc_client(&self) -> &Self::RpcClient {
        &self.rpc_client
    }

    fn event_handler(&self) -> Option<&Self::EventHandler> {
        self.event_handler.as_ref()
    }

    fn unwrap(
        self,
    ) -> (
        Self::Signer,
        Self::Storage,
        Self::RpcClient,
        Option<Self::EventHandler>,
    ) {
        (
            self.signer,
            self.storage,
            self.rpc_client,
            self.event_handler,
        )
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl<S, T, C, E> WithTransaction for StagBuilder<S, T, C, E>
where
    S: Base + Clone,
    T: TransactionProvider,
    C: Base + Clone,
    E: EventHandler + Clone,
{
    type TransactionContext = StagBuilder<S, T::Transaction, C, E>;

    async fn with_transaction(&self) -> Result<Self::TransactionContext> {
        Ok(StagBuilder {
            signer: self.signer.clone(),
            storage: self.storage.transaction().await?,
            rpc_client: self.rpc_client.clone(),
            event_handler: self.event_handler.clone(),
        })
    }
}
