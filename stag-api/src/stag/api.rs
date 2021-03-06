use anyhow::Result;
use primitive_types::U256;
use rust_decimal::Decimal;

use crate::{
    event::NoopEventHandler,
    service::{
        add_chain, close_channel, connect, create_ica_channel, create_transfer_channel,
        get_all_chains, get_balance, get_chain, get_history, get_ibc_balance, get_ibc_denom,
        get_ica_address, get_public_keys, ica, transfer, update_signer,
    },
    signer::{NoopSigner, Signer, SignerConfig},
    storage::{NoopStorage, Storage, TransactionProvider},
    tendermint::{JsonRpcClient, NoopRpcClient},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
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
{
    /// Update signer configuration
    pub fn set_signer<T>(&mut self, signer: T) -> Result<()>
    where
        T: SignerConfig,
        C::Signer: From<T::Signer>,
    {
        self.context.set_signer(signer.into_signer()?.into());
        Ok(())
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
    pub async fn get_ibc_denom(
        &self,
        chain_id: &ChainId,
        port_id: &PortId,
        denom: &Identifier,
    ) -> Result<String> {
        get_ibc_denom(&self.context, chain_id, port_id, denom).await
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
    /// Get on-chain balance of given IBC denom
    pub async fn get_ibc_balance(
        &self,
        chain_id: &ChainId,
        port_id: &PortId,
        denom: &Identifier,
    ) -> Result<Decimal> {
        get_ibc_balance(&self.context, chain_id, port_id, denom).await
    }

    /// Get on-chain balance of given denom
    pub async fn get_balance(&self, chain_id: &ChainId, denom: &Identifier) -> Result<Decimal> {
        get_balance(&self.context, chain_id, denom).await
    }

    /// Get on-chain ICA (Interchain Account) address
    pub async fn get_ica_address(&self, chain_id: &ChainId) -> Result<String> {
        get_ica_address(&self.context, chain_id).await
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

    /// Creates a new transfer channel
    pub async fn create_transfer_channel(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        memo: String,
    ) -> Result<()> {
        create_transfer_channel(&self.context, chain_id, request_id, memo).await
    }

    /// Creates a new ICA (Interchain Accounts) channel
    pub async fn create_ica_channel(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        memo: String,
    ) -> Result<()> {
        create_ica_channel(&self.context, chain_id, request_id, memo).await
    }

    /// Closes the channel with given port id
    pub async fn close_channel(
        &self,
        chain_id: ChainId,
        port_id: &PortId,
        request_id: Option<String>,
        memo: String,
    ) -> Result<()> {
        close_channel(&self.context, chain_id, port_id, request_id, memo).await
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
        transfer::mint_tokens(
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

    /// Burns tokens on given chain
    pub async fn burn(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        transfer::burn_tokens(&self.context, chain_id, request_id, amount, denom, memo).await
    }

    /// Send tokens from ICA (Interchain Account) on host chain
    pub async fn ica_send(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        to_address: String,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        ica::bank::send(
            &self.context,
            chain_id,
            request_id,
            to_address,
            amount,
            denom,
            memo,
        )
        .await
    }

    /// Delegate tokens from ICA (Interchain Account) on host chain to given validator address
    pub async fn ica_delegate(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        validator_address: String,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        ica::staking::delegate(
            &self.context,
            chain_id,
            request_id,
            validator_address,
            amount,
            denom,
            memo,
        )
        .await
    }

    /// Un-delegate tokens to ICA (Interchain Account) on host chain from given validator address
    pub async fn ica_undelegate(
        &self,
        chain_id: ChainId,
        request_id: Option<String>,
        validator_address: String,
        amount: U256,
        denom: Identifier,
        memo: String,
    ) -> Result<String> {
        ica::staking::undelegate(
            &self.context,
            chain_id,
            request_id,
            validator_address,
            amount,
            denom,
            memo,
        )
        .await
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
