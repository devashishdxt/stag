use std::{collections::HashSet, rc::Rc};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use primitive_types::U256;
use prost::Message;
use rexie::{
    Direction, Index, KeyRange, ObjectStore, Rexie, Transaction as RexieTransaction,
    TransactionMode,
};
use tendermint::node::Id as NodeId;

use crate::{
    time_util::now_utc,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainKeyRequest, ChainState},
        ibc_data::IbcData,
        ics::core::ics24_host::{
            identifier::{ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId},
            path::{ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath},
        },
        operation::{Operation, OperationRequest, OperationType},
        proto_util::proto_encode,
    },
};

use super::{Storage, Transaction, TransactionProvider};

pub const CHAIN_STATE_STORE_NAME: &str = "chain_state";
pub const CHAIN_KEY_STORE_NAME: &str = "chain_key";
pub const IBC_DATA_STORE_NAME: &str = "ibc_data";
pub const OPERATIONS_STORE_NAME: &str = "operations";

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
                    .add_index(Index::new("chain_id", "chainId"))
                    .add_index(Index::new("address", "address")),
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

        Ok(IndexedDbTransaction {
            transaction: rexie_transaction,
        })
    }
}

pub struct IndexedDbTransaction {
    transaction: RexieTransaction,
}

impl IndexedDbTransaction {
    async fn add_ibc_data(&self, ibc_data: &IbcData) -> Result<()> {
        let store = self
            .transaction
            .store(IBC_DATA_STORE_NAME)
            .map_err(|err| anyhow!("error when getting ibc_data object store: {}", err))?;

        store
            .add(
                &serde_wasm_bindgen::to_value(&ibc_data)
                    .map_err(|err| anyhow!("error when serializing ibc_data: {}", err))?,
                None,
            )
            .await
            .map_err(|err| anyhow!("error when adding value in ibc_data object store: {}", err))
            .map(|_| ())
    }

    async fn update_ibc_data(&self, ibc_data: &IbcData) -> Result<()> {
        let store = self
            .transaction
            .store(IBC_DATA_STORE_NAME)
            .map_err(|err| anyhow!("error when getting ibc_data object store: {}", err))?;

        store
            .put(
                &serde_wasm_bindgen::to_value(&ibc_data)
                    .map_err(|err| anyhow!("error when serializing ibc_data: {}", err))?,
                None,
            )
            .await
            .map_err(|err| anyhow!("error when putting value in ibc_data object store: {}", err))
            .map(|_| ())
    }

    async fn get_ibc_data(&self, path: &str) -> Result<Option<IbcData>> {
        let store = self
            .transaction
            .store(IBC_DATA_STORE_NAME)
            .map_err(|err| anyhow!("error when getting ibc_data object store: {}", err))?;

        let ibc_data = store
            .get(
                &serde_wasm_bindgen::to_value(path)
                    .map_err(|err| anyhow!("error when serializing client_id: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting value from ibc_data object store: {}",
                    err
                )
            })?;

        serde_wasm_bindgen::from_value(ibc_data)
            .map_err(|err| anyhow!("error when deserializing ibc_data: {}", err))
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl Transaction for IndexedDbTransaction {
    async fn done(self) -> Result<()> {
        self.transaction
            .commit()
            .await
            .map_err(|err| anyhow!("error when committing indexed db transaction: {}", err))
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl Storage for IndexedDbTransaction {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let current_time = now_utc()?;

        let chain_state = ChainState {
            id: chain_id,
            node_id,
            config: chain_config,
            consensus_timestamp: current_time,
            sequence: 1,
            packet_sequence: 1,
            connection_details: None,
            created_at: current_time,
            updated_at: current_time,
        };

        let store = self
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        store
            .put(
                &serde_wasm_bindgen::to_value(&chain_state)
                    .map_err(|err| anyhow!("error when serializing chain_state: {}", err))?,
                None,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in chain_state object store: {}",
                    err
                )
            })
            .map(|_| ())
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let store = self
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        let value = store
            .get(
                &serde_wasm_bindgen::to_value(chain_id)
                    .map_err(|err| anyhow!("error when serializing chain_state: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting value from chain_state object store: {}",
                    err
                )
            })?;

        serde_wasm_bindgen::from_value(value)
            .map_err(|err| anyhow!("error when deserializing chain_state: {}", err))
    }

    async fn update_chain_state(&self, chain_state: &ChainState) -> Result<()> {
        let store = self
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        store
            .put(
                &serde_wasm_bindgen::to_value(chain_state)
                    .map_err(|err| anyhow!("error when serializing chain_state: {}", err))?,
                None,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in chain_state object store: {}",
                    err
                )
            })
            .map(|_| ())
    }

    async fn get_all_chain_states(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>> {
        let store = self
            .transaction
            .store(CHAIN_STATE_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_state object store: {}", err))?;

        let values = store
            .get_all(None, limit, offset, None)
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting values from chain_state object store: {}",
                    err
                )
            })?;

        values
            .into_iter()
            .map(|(_, js_value)| {
                serde_wasm_bindgen::from_value(js_value)
                    .map_err(|err| anyhow!("error when deserializing chain_state: {}", err))
            })
            .collect()
    }

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        let store = self
            .transaction
            .store(CHAIN_KEY_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_key object store: {}", err))?;

        let chain_key = ChainKeyRequest {
            chain_id,
            public_key,
            created_at: now_utc()?,
        };

        store
            .add(
                &serde_wasm_bindgen::to_value(&chain_key)
                    .map_err(|err| anyhow!("error when serializing chain_key: {}", err))?,
                None,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in chain_key object store: {}",
                    err
                )
            })
            .map(|_| ())
    }

    async fn get_chain_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>> {
        let store = self
            .transaction
            .store(CHAIN_KEY_STORE_NAME)
            .map_err(|err| anyhow!("error when getting chain_key object store: {}", err))?;

        let js_chain_id = serde_wasm_bindgen::to_value(&chain_id)
            .map_err(|err| anyhow!("error when serializing chain_id: {}", err))?;

        store
            .get_all(
                Some(
                    &KeyRange::only(&js_chain_id)
                        .map_err(|err| anyhow!("unable to generate keyrange: {}", err))?,
                ),
                limit,
                offset,
                Some(Direction::Prev),
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting all chain keys for chain id [{}]: {}",
                    chain_id,
                    err
                )
            })?
            .into_iter()
            .map(|pair| pair.1)
            .map(|value| {
                serde_wasm_bindgen::from_value(value)
                    .map_err(|err| anyhow!("error when deserializing chain_key: {}", err))
            })
            .collect()
    }

    async fn add_operation(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        address: &str,
        denom: &Identifier,
        amount: &U256,
        operation_type: OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        let operation = OperationRequest {
            request_id,
            chain_id,
            address,
            denom,
            amount,
            operation_type,
            transaction_hash,
            created_at: now_utc()?,
        };

        let store = self
            .transaction
            .store(OPERATIONS_STORE_NAME)
            .map_err(|err| anyhow!("error when getting operations object store: {}", err))?;

        store
            .add(
                &serde_wasm_bindgen::to_value(&operation)
                    .map_err(|err| anyhow!("error when serializing operation: {}", err))?,
                None,
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when putting value in operation object store: {}",
                    err
                )
            })
            .map(|_| ())
    }

    async fn get_operations(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Operation>> {
        let store = self
            .transaction
            .store(OPERATIONS_STORE_NAME)
            .map_err(|err| anyhow!("error when getting operations object store: {}", err))?;

        let index = store
            .index("chain_id")
            .map_err(|err| anyhow!("error when getting chain id index: {}", err))?;

        let js_chain_id = serde_wasm_bindgen::to_value(chain_id)
            .map_err(|err| anyhow!("error when serializing chain_id: {}", err))?;

        let pairs = index
            .get_all(
                Some(
                    &KeyRange::only(&js_chain_id)
                        .map_err(|err| anyhow!("unable to generate keyrange: {}", err))?,
                ),
                limit,
                offset,
                Some(Direction::Prev),
            )
            .await
            .map_err(|err| {
                anyhow!(
                    "error when getting all operations for chain id [{}]: {}",
                    chain_id,
                    err
                )
            })?;

        pairs
            .into_iter()
            .map(|(_, js_value)| {
                serde_wasm_bindgen::from_value(js_value)
                    .map_err(|err| anyhow!("error when deserializing operation: {}", err))
            })
            .collect()
    }

    async fn add_tendermint_client_state(
        &self,
        client_id: &ClientId,
        client_state: &TendermintClientState,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ClientStatePath::new(client_id).into(),
            data: proto_encode(client_state)?,
        };

        self.add_ibc_data(&ibc_data).await
    }

    async fn get_tendermint_client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<TendermintClientState>> {
        let path: String = ClientStatePath::new(client_id).into();

        let ibc_data: Option<IbcData> = self.get_ibc_data(&path).await?;

        match ibc_data {
            None => Ok(None),
            Some(ibc_data) => TendermintClientState::decode(ibc_data.data.as_slice())
                .context("error when deserializing tendermint client state")
                .map(Some),
        }
    }

    async fn add_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: &TendermintConsensusState,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ConsensusStatePath::new(client_id, height).into(),
            data: proto_encode(consensus_state)?,
        };

        self.add_ibc_data(&ibc_data).await
    }

    async fn get_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        let path: String = ConsensusStatePath::new(client_id, height).into();

        let ibc_data: Option<IbcData> = self.get_ibc_data(&path).await?;

        match ibc_data {
            None => Ok(None),
            Some(ibc_data) => TendermintConsensusState::decode(ibc_data.data.as_slice())
                .context("error when deserializing tendermint consensus state")
                .map(Some),
        }
    }

    async fn add_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ConnectionPath::new(connection_id).into(),
            data: proto_encode(connection)?,
        };

        self.add_ibc_data(&ibc_data).await
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        let path: String = ConnectionPath::new(connection_id).into();

        let ibc_data: Option<IbcData> = self.get_ibc_data(&path).await?;

        match ibc_data {
            None => Ok(None),
            Some(ibc_data) => ConnectionEnd::decode(ibc_data.data.as_slice())
                .context("error when deserializing connection")
                .map(Some),
        }
    }

    async fn update_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ConnectionPath::new(connection_id).into(),
            data: proto_encode(connection)?,
        };

        self.update_ibc_data(&ibc_data).await
    }

    async fn add_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ChannelPath::new(port_id, channel_id).into(),
            data: proto_encode(channel)?,
        };

        self.add_ibc_data(&ibc_data).await
    }

    async fn get_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Option<Channel>> {
        let path: String = ChannelPath::new(port_id, channel_id).into();

        let ibc_data: Option<IbcData> = self.get_ibc_data(&path).await?;

        match ibc_data {
            None => Ok(None),
            Some(ibc_data) => Channel::decode(ibc_data.data.as_slice())
                .context("error when deserializing connection")
                .map(Some),
        }
    }

    async fn update_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let ibc_data = IbcData {
            path: ChannelPath::new(port_id, channel_id).into(),
            data: proto_encode(channel)?,
        };

        self.update_ibc_data(&ibc_data).await
    }

    async fn delete(self) -> Result<()> {
        Err(anyhow!("cannot delete the storage from a transaction"))
    }
}

impl TransactionProvider for IndexedDbStorage {
    type Transaction = Self;

    fn transaction(&self) -> Result<Self::Transaction> {
        Ok(self.clone())
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl Transaction for IndexedDbStorage {
    async fn done(self) -> Result<()> {
        Ok(())
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
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
        address: &str,
        denom: &Identifier,
        amount: &U256,
        operation_type: OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        let transaction = self.get_transaction(&["add_operation"])?;

        transaction
            .add_operation(
                request_id,
                chain_id,
                address,
                denom,
                amount,
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

    async fn delete(self) -> Result<()> {
        let name = self.rexie.name();

        Rexie::delete(&name)
            .await
            .map_err(|err| anyhow!("unable to delete indexed db database: {}", err))
    }
}
