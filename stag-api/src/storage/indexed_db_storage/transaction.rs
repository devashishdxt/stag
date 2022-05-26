use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use prost::Message;
use rexie::{Direction, KeyRange, Transaction as RexieTransaction};
use tendermint::node::Id as NodeId;

use crate::{
    storage::{Storage, Transaction},
    time_util::now_utc,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ibc_data::IbcData,
        ics::core::ics24_host::{
            identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
            path::{ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath},
        },
        operation::{Operation, OperationType},
        proto_util::proto_encode,
    },
};

use super::{
    types::{ChainKeyRequest, OperationRequest},
    CHAIN_KEY_STORE_NAME, CHAIN_STATE_STORE_NAME, IBC_DATA_STORE_NAME, OPERATIONS_STORE_NAME,
};

pub struct IndexedDbTransaction {
    transaction: RexieTransaction,
}

impl IndexedDbTransaction {
    pub fn new(transaction: RexieTransaction) -> Self {
        Self { transaction }
    }

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

#[async_trait(?Send)]
impl Transaction for IndexedDbTransaction {
    async fn done(self) -> Result<()> {
        self.transaction
            .done()
            .await
            .map_err(|err| anyhow!("error when committing indexed db transaction: {}", err))
    }
}

#[async_trait(?Send)]
impl Storage for IndexedDbTransaction {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let current_time = now_utc();

        let chain_state = ChainState {
            id: chain_id,
            node_id,
            config: chain_config,
            consensus_timestamp: current_time,
            sequence: 1,
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
            created_at: now_utc(),
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

        let index = store.index("chain_id").map_err(|err| {
            anyhow!(
                "error when getting index from chain_key object store: {}",
                err
            )
        })?;

        let js_chain_id = serde_wasm_bindgen::to_value(&chain_id)
            .map_err(|err| anyhow!("error when serializing chain_id: {}", err))?;

        index
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
        port_id: &PortId,
        operation_type: &OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        let operation = OperationRequest {
            request_id,
            chain_id,
            port_id,
            operation_type,
            transaction_hash,
            created_at: now_utc(),
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
