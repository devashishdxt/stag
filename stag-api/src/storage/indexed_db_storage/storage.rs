use std::{collections::HashSet, rc::Rc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use rexie::{Index, ObjectStore, Rexie, TransactionMode};
use tendermint::node::Id as NodeId;

use crate::{
    storage::{Storage, Transaction, TransactionProvider},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
        operation::{Operation, OperationType},
    },
};

use super::{
    IndexedDbTransaction, CHAIN_KEY_STORE_NAME, CHAIN_STATE_STORE_NAME, IBC_DATA_STORE_NAME,
    OPERATIONS_STORE_NAME,
};

#[derive(Clone)]
pub struct IndexedDbStorage {
    rexie: Rc<Rexie>,
}

impl IndexedDbStorage {
    pub async fn new(name: &str) -> Result<Self> {
        let rexie = Rexie::builder(name)
            .version(1)
            .add_object_store(ObjectStore::new(CHAIN_STATE_STORE_NAME).key_path("id"))
            .add_object_store(
                ObjectStore::new(CHAIN_KEY_STORE_NAME)
                    .key_path("id")
                    .auto_increment(true)
                    .add_index(Index::new("chain_id", "chainId")),
            )
            .add_object_store(ObjectStore::new(IBC_DATA_STORE_NAME).key_path("path"))
            .add_object_store(
                ObjectStore::new(OPERATIONS_STORE_NAME)
                    .key_path("id")
                    .auto_increment(true)
                    .add_index(Index::new("chain_id", "chainId")),
            )
            .build()
            .await
            .map_err(|err| anyhow!("error when opening indexed db: {}", err))?;

        Ok(Self {
            rexie: Rc::new(rexie),
        })
    }

    fn get_transaction(&self, access_points: &[&str]) -> Result<IndexedDbTransaction> {
        let mut store_names = HashSet::new();
        let mut is_write = false;

        for access_point in access_points {
            let (store_name, write_required) = match *access_point {
                "add_chain_state" => (CHAIN_STATE_STORE_NAME, true),
                "get_chain_state" => (CHAIN_STATE_STORE_NAME, false),
                "update_chain_state" => (CHAIN_STATE_STORE_NAME, true),
                "get_all_chain_states" => (CHAIN_STATE_STORE_NAME, false),
                "add_chain_key" => (CHAIN_KEY_STORE_NAME, true),
                "get_chain_keys" => (CHAIN_KEY_STORE_NAME, false),
                "add_operation" => (OPERATIONS_STORE_NAME, true),
                "get_operations" => (OPERATIONS_STORE_NAME, false),
                "add_tendermint_client_state" => (IBC_DATA_STORE_NAME, true),
                "get_tendermint_client_state" => (IBC_DATA_STORE_NAME, false),
                "add_tendermint_consensus_state" => (IBC_DATA_STORE_NAME, true),
                "get_tendermint_consensus_state" => (IBC_DATA_STORE_NAME, false),
                "add_connection" => (IBC_DATA_STORE_NAME, true),
                "get_connection" => (IBC_DATA_STORE_NAME, false),
                "update_connection" => (IBC_DATA_STORE_NAME, true),
                "add_channel" => (IBC_DATA_STORE_NAME, true),
                "get_channel" => (IBC_DATA_STORE_NAME, false),
                "update_channel" => (IBC_DATA_STORE_NAME, true),
                "add_ica_address" => (IBC_DATA_STORE_NAME, true),
                "get_ica_address" => (IBC_DATA_STORE_NAME, false),
                "update_ica_address" => (IBC_DATA_STORE_NAME, true),
                _ => return Err(anyhow!("unknown access point: {}", access_point)),
            };

            if write_required {
                is_write = true;
            }

            store_names.insert(store_name.to_string());
        }

        let mode = if is_write {
            TransactionMode::ReadWrite
        } else {
            TransactionMode::ReadOnly
        };

        let store_names: Vec<String> = store_names.into_iter().collect();

        let rexie_transaction = self
            .rexie
            .transaction(store_names.as_slice(), mode)
            .map_err(|err| anyhow!("error when opening indexed db transaction: {}", err))?;

        Ok(IndexedDbTransaction::new(rexie_transaction))
    }
}

#[async_trait(?Send)]
impl TransactionProvider for IndexedDbStorage {
    type Transaction = Self;

    async fn transaction(&self) -> Result<Self::Transaction> {
        Ok(self.clone())
    }
}

#[async_trait(?Send)]
impl Transaction for IndexedDbStorage {
    async fn done(self) -> Result<()> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl Storage for IndexedDbStorage {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_chain_state"])?;

        transaction
            .add_chain_state(chain_id, node_id, chain_config)
            .await?;

        transaction.done().await
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let transaction = self.get_transaction(&["get_chain_state"])?;

        let result = transaction.get_chain_state(chain_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn update_chain_state(&self, chain_state: &ChainState) -> Result<()> {
        let transaction = self.get_transaction(&["update_chain_state"])?;

        transaction.update_chain_state(chain_state).await?;

        transaction.done().await
    }

    async fn get_all_chain_states(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>> {
        let transaction = self.get_transaction(&["get_all_chain_states"])?;

        let result = transaction.get_all_chain_states(limit, offset).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        let transaction = self.get_transaction(&["add_chain_key"])?;

        transaction.add_chain_key(chain_id, public_key).await?;

        transaction.done().await
    }

    async fn get_chain_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>> {
        let transaction = self.get_transaction(&["get_chain_keys"])?;

        let result = transaction.get_chain_keys(chain_id, limit, offset).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_operation(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        port_id: &PortId,
        operation_type: &OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_operation"])?;

        transaction
            .add_operation(
                request_id,
                chain_id,
                port_id,
                operation_type,
                transaction_hash,
            )
            .await?;

        transaction.done().await
    }

    async fn get_operations(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Operation>> {
        let transaction = self.get_transaction(&["get_operations"])?;

        let result = transaction.get_operations(chain_id, limit, offset).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_tendermint_client_state(
        &self,
        client_id: &ClientId,
        client_state: &TendermintClientState,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_tendermint_client_state"])?;

        transaction
            .add_tendermint_client_state(client_id, client_state)
            .await?;

        transaction.done().await
    }

    async fn get_tendermint_client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<TendermintClientState>> {
        let transaction = self.get_transaction(&["get_tendermint_client_state"])?;

        let result = transaction.get_tendermint_client_state(client_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: &TendermintConsensusState,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_tendermint_consensus_state"])?;

        transaction
            .add_tendermint_consensus_state(client_id, height, consensus_state)
            .await?;

        transaction.done().await
    }

    async fn get_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        let transaction = self.get_transaction(&["get_tendermint_consensus_state"])?;

        let result = transaction
            .get_tendermint_consensus_state(client_id, height)
            .await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn add_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_connection"])?;

        transaction
            .add_connection(connection_id, connection)
            .await?;

        transaction.done().await
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        let transaction = self.get_transaction(&["get_connection"])?;

        let result = transaction.get_connection(connection_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn update_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["update_connection"])?;

        transaction
            .update_connection(connection_id, connection)
            .await?;

        transaction.done().await
    }

    async fn add_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_channel"])?;

        transaction
            .add_channel(port_id, channel_id, channel)
            .await?;

        transaction.done().await
    }

    async fn get_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Option<Channel>> {
        let transaction = self.get_transaction(&["get_channel"])?;

        let result = transaction.get_channel(port_id, channel_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    async fn update_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["update_channel"])?;

        transaction
            .update_channel(port_id, channel_id, channel)
            .await?;

        transaction.done().await
    }

    async fn add_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
        address: &str,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_ica_address"])?;

        transaction
            .add_ica_address(connection_id, port_id, address)
            .await?;

        transaction.done().await
    }

    /// Gets ICA address from the storage
    async fn get_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
    ) -> Result<Option<String>> {
        let transaction = self.get_transaction(&["get_ica_address"])?;

        let result = transaction.get_ica_address(connection_id, port_id).await?;

        transaction.done().await?;

        Ok(result)
    }

    /// Updates ICA address in the storage
    async fn update_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
        address: &str,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["update_ica_address"])?;

        transaction
            .update_ica_address(connection_id, port_id, address)
            .await?;

        transaction.done().await
    }

    async fn delete(self) -> Result<()> {
        let name = self.rexie.name();

        Rexie::delete(&name)
            .await
            .map_err(|err| anyhow!("unable to delete indexed db database: {}", err))
    }
}
