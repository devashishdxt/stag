use anyhow::Result;
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use tendermint::node::Id as NodeId;

use crate::{
    trait_util::Base,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
        operation::{Operation, OperationType},
    },
};

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
/// Trait that must be implemented by all database transaction types
pub trait Transaction: Storage {
    /// Commit the transaction
    async fn done(self) -> Result<()>;
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
/// Trait that must be implemented by all database storage types
pub trait TransactionProvider: Storage {
    /// Type of transaction for current storage type
    type Transaction: Transaction;

    /// Create a new transaction
    async fn transaction(&self) -> Result<Self::Transaction>;
}

/// Trait that must be implemented by all database storage types
#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Storage: Base {
    /// Adds a new chain to the storage
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()>;

    /// Gets a chain from the storage
    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>>;

    /// Updates a chain in the storage
    async fn update_chain_state(&self, chain_state: &ChainState) -> Result<()>;

    /// Get all chains from the storage
    async fn get_all_chain_states(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>>;

    /// Adds a new public key for a given chain to the storage
    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()>;

    /// Gets all public keys for a given chain from the storage
    async fn get_chain_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>>;

    /// Adds a new IBC operation to the storage
    async fn add_operation(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        port_id: &PortId,
        operation_type: &OperationType,
        transaction_hash: &str,
    ) -> Result<()>;

    /// Gets all IBC operations from the storage for a given chain
    async fn get_operations(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Operation>>;

    /// Adds tendermint client state to the storage
    async fn add_tendermint_client_state(
        &self,
        client_id: &ClientId,
        client_state: &TendermintClientState,
    ) -> Result<()>;

    /// Gets tendermint client state from the storage
    async fn get_tendermint_client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<TendermintClientState>>;

    /// Adds tendermint consensus state to the storage
    async fn add_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: &TendermintConsensusState,
    ) -> Result<()>;

    /// Gets tendermint consensus state from the storage
    async fn get_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>>;

    /// Adds connection to the storage
    async fn add_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()>;

    /// Gets connection from the storage
    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>>;

    /// Updates connection in the storage
    async fn update_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()>;

    /// Adds channel to the storage
    async fn add_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()>;

    /// Gets channel from the storage
    async fn get_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Option<Channel>>;

    /// Updates channel in the storage
    async fn update_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()>;

    /// Delete the storage (should only be used for testing)
    async fn delete(self) -> Result<()>;
}
