use anyhow::Result;
use primitive_types::U256;
use rust_decimal::Decimal;

use crate::{
    event::NoopEventHandler,
    service::{
        add_chain, burn_tokens, connect, get_all_chains, get_balance, get_chain, get_history,
        get_ibc_denom, get_public_keys, mint_tokens, update_signer,
    },
    signer::{NoopSigner, Signer},
    storage::{NoopStorage, Storage, TransactionProvider},
    tendermint::{JsonRpcClient, NoopRpcClient},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, Identifier},
        operation::Operation,
        public_key::PublicKey,
    },
};

use super::{StagBuilder, StagContext, WithTransaction};

/// Stag API
pub struct Stag<C> {
    context: C,
}

impl<T> From<T> for Stag<T>
where
    T: StagContext,
{
    fn from(context: T) -> Self {
        Self { context }
    }
}

impl Stag<StagBuilder<NoopSigner, NoopStorage, NoopRpcClient, NoopEventHandler>> {
    /// Creates a builder for Stag API
    pub fn builder() -> StagBuilder<NoopSigner, NoopStorage, NoopRpcClient, NoopEventHandler> {
        StagBuilder::default()
    }
}

impl<C> Stag<C>
where
    C: StagContext,
    C::Storage: Storage,
{
    /// Gets current stored state for a given chain
    pub async fn get_chain(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        get_chain(&self.context, chain_id).await
    }

    /// Gets all the stored chain states
    pub async fn get_all_chains(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>> {
        get_all_chains(&self.context, limit, offset).await
    }

    /// Gets the final denom of a token on solo machine after sending it on given chain
    pub async fn get_ibc_denom(&self, chain_id: &ChainId, denom: &Identifier) -> Result<String> {
        get_ibc_denom(&self.context, chain_id, denom).await
    }

    /// Get all the historical public keys associated with solo machine client on given chain
    pub async fn get_public_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>> {
        get_public_keys(&self.context, chain_id, limit, offset).await
    }

    /// Gets transaction history of given chain
    pub async fn get_history(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Operation>> {
        get_history(&self.context, chain_id, limit, offset).await
    }
}

impl<C> Stag<C>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    /// Get on-chain balance of given denom
    pub async fn get_balance(&self, chain_id: &ChainId, denom: &Identifier) -> Result<Decimal> {
        get_balance(&self.context, chain_id, denom).await
    }
}

impl<C> Stag<C>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer + Clone,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient + Clone,
    C::EventHandler: Clone,
{
    /// Adds metadata of a given chain
    pub async fn add_chain(&self, chain_config: &ChainConfig) -> Result<ChainId> {
        add_chain(&self.context, chain_config).await
    }

    /// Establishes connection with given chain
    pub async fn connect(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        memo: String,
        force: bool,
    ) -> Result<()> {
        connect(&self.context, chain_id, request_id, memo, force).await
    }

    /// Mints tokens on given chain
    pub async fn mint(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        receiver: Option<String>,
        memo: String,
    ) -> Result<String> {
        mint_tokens(
            &self.context,
            chain_id,
            request_id,
            amount,
            denom,
            receiver,
            memo,
        )
        .await
    }

    /// Updates signer for future IBC transactions
    pub async fn update_signer(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        new_public_key: PublicKey,
        memo: String,
    ) -> Result<()> {
        update_signer(&self.context, chain_id, request_id, new_public_key, memo).await
    }
}

impl<C> Stag<C>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: JsonRpcClient,
{
    /// Burns tokens on given chain
    pub async fn burn(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        burn_tokens(&self.context, chain_id, request_id, amount, denom, memo).await
    }
}

impl<C> Stag<C>
where
    C: StagContext,
    C::Storage: Storage,
{
    /// Clears and deletes the storage (should only be used for testing)
    pub async fn clear(self) -> Result<()> {
        let (_, storage, _, _) = self.context.unwrap();
        storage.delete().await
    }
}
