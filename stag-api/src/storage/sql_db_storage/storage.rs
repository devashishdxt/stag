use anyhow::{Context, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Pool,
};
use tendermint::node::Id as NodeId;

use crate::{
    storage::{Storage, TransactionProvider},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
        operation::{Operation, OperationType},
    },
};

use super::{executor, Db, SqlDbTransaction};

#[cfg(feature = "sqlite-storage")]
const MIGRATOR: Migrator = sqlx::migrate!("./migrations/sqlite");

pub struct SqlDbStorage {
    uri: String,
    pool: Pool<Db>,
}

impl SqlDbStorage {
    pub async fn new(uri: String) -> Result<Self> {
        if !Db::database_exists(&uri).await? {
            Db::create_database(&uri).await?;
        }

        let pool = Pool::connect(&uri)
            .await
            .context("unable to connect to database")?;

        MIGRATOR.run(&pool).await?;

        Ok(Self { uri, pool })
    }
}

#[async_trait]
impl TransactionProvider for SqlDbStorage {
    type Transaction = SqlDbTransaction;

    async fn transaction(&self) -> Result<Self::Transaction> {
        let transaction = self
            .pool
            .begin()
            .await
            .context("unable to start transaction")?;

        Ok(SqlDbTransaction::new(transaction))
    }
}

#[async_trait]
impl Storage for SqlDbStorage {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        executor::add_chain_state(&self.pool, chain_id, node_id, chain_config).await
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        executor::get_chain_state(&self.pool, chain_id).await
    }

    async fn update_chain_state(&self, chain_state: &ChainState) -> Result<()> {
        executor::update_chain_state(&self.pool, chain_state).await
    }

    async fn get_all_chain_states(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>> {
        executor::get_all_chain_states(&self.pool, limit, offset).await
    }

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        executor::add_chain_key(&self.pool, chain_id, public_key).await
    }

    async fn get_chain_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>> {
        executor::get_chain_keys(&self.pool, chain_id, limit, offset).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_operation(
        &self,
        request_id: Option<&str>,
        chain_id: &ChainId,
        port_id: &PortId,
        operation_type: &OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        executor::add_operation(
            &self.pool,
            request_id,
            chain_id,
            port_id,
            operation_type,
            transaction_hash,
        )
        .await
    }

    async fn get_operations(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Operation>> {
        executor::get_operations(&self.pool, chain_id, limit, offset).await
    }

    async fn add_tendermint_client_state(
        &self,
        client_id: &ClientId,
        client_state: &TendermintClientState,
    ) -> Result<()> {
        executor::add_tendermint_client_state(&self.pool, client_id, client_state).await
    }

    async fn get_tendermint_client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<TendermintClientState>> {
        executor::get_tendermint_client_state(&self.pool, client_id).await
    }

    async fn add_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: &TendermintConsensusState,
    ) -> Result<()> {
        executor::add_tendermint_consensus_state(&self.pool, client_id, height, consensus_state)
            .await
    }

    async fn get_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        executor::get_tendermint_consensus_state(&self.pool, client_id, height).await
    }

    async fn add_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        executor::add_connection(&self.pool, connection_id, connection).await
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        executor::get_connection(&self.pool, connection_id).await
    }

    async fn update_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        executor::update_connection(&self.pool, connection_id, connection).await
    }

    async fn add_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        executor::add_channel(&self.pool, port_id, channel_id, channel).await
    }

    async fn get_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Option<Channel>> {
        executor::get_channel(&self.pool, port_id, channel_id).await
    }

    async fn update_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        executor::update_channel(&self.pool, port_id, channel_id, channel).await
    }

    async fn add_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
        address: &str,
    ) -> Result<()> {
        executor::add_ica_address(&self.pool, connection_id, port_id, address).await
    }

    async fn get_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
    ) -> Result<Option<String>> {
        executor::get_ica_address(&self.pool, connection_id, port_id).await
    }

    async fn update_ica_address(
        &self,
        connection_id: &ConnectionId,
        port_id: &PortId,
        address: &str,
    ) -> Result<()> {
        executor::update_ica_address(&self.pool, connection_id, port_id, address).await
    }

    async fn delete(self) -> Result<()> {
        self.pool.close().await;

        Db::drop_database(&self.uri)
            .await
            .context("unable to drop database")
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use primitive_types::U256;

    use crate::types::{chain_state::Fee, ics::core::ics24_host::identifier::Identifier};

    use super::*;

    const URI: &str = "sqlite::memory:";

    #[tokio::test]
    async fn test_new_storage() {
        assert!(SqlDbStorage::new(URI.to_owned()).await.is_ok());
    }

    #[tokio::test]
    async fn test_chain_state() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let chain_id: ChainId = "test-1".parse().unwrap();
        let node_id = NodeId::new([0; 20]);
        let chain_config = ChainConfig {
            grpc_addr: "http://0.0.0.0:9090".parse().unwrap(),
            rpc_addr: "http://0.0.0.0:26657".parse().unwrap(),
            fee: Fee {
                amount: "1000".parse().unwrap(),
                denom: "atom".parse().unwrap(),
                gas_limit: 300000,
            },
            trust_level: "1/3".parse().unwrap(),
            trusting_period: Duration::from_secs(336 * 60 * 60),
            max_clock_drift: Duration::from_secs(3),
            rpc_timeout: Duration::from_secs(60),
            diversifier: "stag".to_owned(),
            trusted_height: 1,
            trusted_hash: [0; 32],
            packet_timeout_height_offset: 10,
        };

        // Add a new chain state
        assert!(storage
            .add_chain_state(chain_id.clone(), node_id, chain_config.clone())
            .await
            .is_ok());

        // Should return `None` chain state with invalid chain id
        let chain_state = storage.get_chain_state(&"test-2".parse().unwrap()).await;
        assert!(chain_state.is_ok(), "error: {:?}", chain_state.unwrap_err());
        let chain_state = chain_state.unwrap();
        assert!(chain_state.is_none());

        // Should return chain state with valid chain id
        let chain_state = storage.get_chain_state(&chain_id).await;
        assert!(chain_state.is_ok(), "error: {:?}", chain_state.unwrap_err());
        let chain_state = chain_state.unwrap();
        assert!(chain_state.is_some());
        let mut chain_state = chain_state.unwrap();

        assert_eq!(chain_state.id, chain_id);
        assert_eq!(chain_state.node_id, node_id);
        assert_eq!(chain_state.config, chain_config);
        assert_eq!(chain_state.sequence, 1);

        // Update chain state
        chain_state.sequence += 1;

        assert!(storage.update_chain_state(&chain_state).await.is_ok());

        // Check updated chain state
        let chain_state = storage.get_chain_state(&chain_id).await;
        assert!(chain_state.is_ok(), "error: {:?}", chain_state.unwrap_err());
        let chain_state = chain_state.unwrap();
        assert!(chain_state.is_some());
        let chain_state = chain_state.unwrap();

        assert_eq!(chain_state.id, chain_id);
        assert_eq!(chain_state.node_id, node_id);
        assert_eq!(chain_state.config, chain_config);
        assert_eq!(chain_state.sequence, 2);

        // Get all chain states should return one chain state
        let chain_states = storage.get_all_chain_states(None, None).await;
        assert!(
            chain_states.is_ok(),
            "error: {:?}",
            chain_states.unwrap_err()
        );
        let chain_states = chain_states.unwrap();

        assert_eq!(chain_states.len(), 1);
        assert_eq!(chain_states[0].id, chain_id);
        assert_eq!(chain_states[0].node_id, node_id);
        assert_eq!(chain_states[0].config, chain_config);
        assert_eq!(chain_states[0].sequence, 2);

        // Get all chain states should not return any values when limit is zero
        let chain_states = storage.get_all_chain_states(Some(0), None).await;
        assert!(
            chain_states.is_ok(),
            "error: {:?}",
            chain_states.unwrap_err()
        );
        let chain_states = chain_states.unwrap();

        assert!(chain_states.is_empty());

        // Get all chain states should not return any values when offset is one
        let chain_states = storage.get_all_chain_states(Some(1), Some(1)).await;
        assert!(
            chain_states.is_ok(),
            "error: {:?}",
            chain_states.unwrap_err()
        );
        let chain_states = chain_states.unwrap();

        assert!(chain_states.is_empty());

        // Offset cannot be set without limit
        let chain_states = storage.get_all_chain_states(None, Some(1)).await;
        assert!(chain_states.is_err());
    }

    #[tokio::test]
    async fn test_chain_key() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let chain_id: ChainId = "test-1".parse().unwrap();

        // Add multiple public keys for a chain
        assert!(storage
            .add_chain_key(&chain_id, "public-key-1")
            .await
            .is_ok());

        assert!(storage
            .add_chain_key(&chain_id, "public-key-2")
            .await
            .is_ok());

        assert!(storage
            .add_chain_key(&chain_id, "public-key-3")
            .await
            .is_ok());

        // Should not return any public keys for invalid chain id
        let public_keys = storage
            .get_chain_keys(&"test-2".parse().unwrap(), None, None)
            .await;
        assert!(public_keys.is_ok(), "error: {:?}", public_keys.unwrap_err());
        let public_keys = public_keys.unwrap();

        assert!(public_keys.is_empty());

        // Should return all public keys for valid chain id
        let public_keys = storage.get_chain_keys(&chain_id, None, None).await;
        assert!(public_keys.is_ok(), "error: {:?}", public_keys.unwrap_err());
        let public_keys = public_keys.unwrap();

        assert_eq!(public_keys.len(), 3);

        // Public keys are sorted by reverse insertion order
        assert_eq!(public_keys[0].public_key, "public-key-3");
        assert_eq!(public_keys[1].public_key, "public-key-2");
        assert_eq!(public_keys[2].public_key, "public-key-1");

        // Should return all public keys for valid chain id with limit and offset
        let public_keys = storage.get_chain_keys(&chain_id, Some(2), Some(1)).await;
        assert!(public_keys.is_ok(), "error: {:?}", public_keys.unwrap_err());
        let public_keys = public_keys.unwrap();

        assert_eq!(public_keys.len(), 2);

        // Public keys are sorted by reverse insertion order
        assert_eq!(public_keys[0].public_key, "public-key-2");
        assert_eq!(public_keys[1].public_key, "public-key-1");
    }

    #[tokio::test]
    async fn test_operation() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let chain_id: ChainId = "test-1".parse().unwrap();
        let port_id = PortId::transfer();
        let denom: Identifier = "denom".parse().unwrap();
        let amount: U256 = 1u8.into();

        // Add multiple operations for a chain
        assert!(storage
            .add_operation(
                None,
                &chain_id,
                &port_id,
                &OperationType::Mint {
                    to: "address-1".to_string(),
                    denom: denom.clone(),
                    amount,
                },
                "transaction-hash-1"
            )
            .await
            .is_ok());

        assert!(storage
            .add_operation(
                None,
                &chain_id,
                &port_id,
                &OperationType::Mint {
                    to: "address-2".to_string(),
                    denom: denom.clone(),
                    amount,
                },
                "transaction-hash-2"
            )
            .await
            .is_ok());

        assert!(storage
            .add_operation(
                None,
                &chain_id,
                &port_id,
                &OperationType::Mint {
                    to: "address-3".to_string(),
                    denom: denom.clone(),
                    amount,
                },
                "transaction-hash-3"
            )
            .await
            .is_ok());

        // Should not return any operations for invalid chain id
        let operations = storage
            .get_operations(&"test-2".parse().unwrap(), None, None)
            .await;
        assert!(operations.is_ok(), "error: {:?}", operations.unwrap_err());
        let operations = operations.unwrap();

        assert!(operations.is_empty());

        // Should return all operations for valid chain id
        let operations = storage.get_operations(&chain_id, None, None).await;
        assert!(operations.is_ok(), "error: {:?}", operations.unwrap_err());
        let operations = operations.unwrap();

        assert_eq!(operations.len(), 3);

        // Operations are sorted by reverse insertion order
        assert!(matches!(
            operations[0].operation_type,
            OperationType::Mint { ref to, .. } if to == "address-3"
        ));
        assert!(matches!(
            operations[1].operation_type,
            OperationType::Mint { ref to, .. } if to == "address-2"
        ));
        assert!(matches!(
            operations[2].operation_type,
            OperationType::Mint { ref to, .. } if to == "address-1"
        ));
    }

    #[tokio::test]
    async fn test_tendermint_client_state() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let client_id: ClientId = "07-tendermint-1".parse().unwrap();
        let client_state = TendermintClientState {
            chain_id: "test-1".to_owned(),
            trust_level: None,
            trusting_period: None,
            unbonding_period: None,
            max_clock_drift: None,
            frozen_height: None,
            latest_height: None,
            proof_specs: vec![],
            upgrade_path: vec![],
            allow_update_after_expiry: false,
            allow_update_after_misbehaviour: false,
        };

        // Add tendermint client states for a chain
        assert!(storage
            .add_tendermint_client_state(&client_id, &client_state)
            .await
            .is_ok());

        // Should not return tendermint client state for invalid client id
        let tendermint_client_state = storage
            .get_tendermint_client_state(&"07-tendermint-2".parse().unwrap())
            .await;
        assert!(
            tendermint_client_state.is_ok(),
            "error: {:?}",
            tendermint_client_state.unwrap_err()
        );
        let tendermint_client_state = tendermint_client_state.unwrap();

        assert!(tendermint_client_state.is_none());

        // Should return tendermint client state for valid client id
        let tendermint_client_state = storage.get_tendermint_client_state(&client_id).await;
        assert!(
            tendermint_client_state.is_ok(),
            "error: {:?}",
            tendermint_client_state.unwrap_err()
        );
        let tendermint_client_state = tendermint_client_state.unwrap();

        assert!(tendermint_client_state.is_some());
        assert_eq!(tendermint_client_state.unwrap(), client_state);
    }

    #[tokio::test]
    async fn test_tendermint_consensus_state() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let client_id: ClientId = "07-tendermint-1".parse().unwrap();
        let height: Height = Height {
            revision_number: 1,
            revision_height: 1,
        };
        let consensus_state = TendermintConsensusState {
            timestamp: None,
            root: None,
            next_validators_hash: vec![],
        };

        // Add tendermint client states for a chain
        assert!(storage
            .add_tendermint_consensus_state(&client_id, &height, &consensus_state)
            .await
            .is_ok());

        // Should not return tendermint consensus state for invalid client id
        let tendermint_consensus_state = storage
            .get_tendermint_consensus_state(&"07-tendermint-2".parse().unwrap(), &height)
            .await;
        assert!(
            tendermint_consensus_state.is_ok(),
            "error: {:?}",
            tendermint_consensus_state.unwrap_err()
        );
        let tendermint_consensus_state = tendermint_consensus_state.unwrap();

        assert!(tendermint_consensus_state.is_none());

        // Should not return tendermint consensus state for invalid height
        let tendermint_consensus_state = storage
            .get_tendermint_consensus_state(
                &client_id,
                &Height {
                    revision_number: 2,
                    revision_height: 2,
                },
            )
            .await;
        assert!(
            tendermint_consensus_state.is_ok(),
            "error: {:?}",
            tendermint_consensus_state.unwrap_err()
        );
        let tendermint_consensus_state = tendermint_consensus_state.unwrap();

        assert!(tendermint_consensus_state.is_none());

        // Should return tendermint consensus state for valid client id and height
        let tendermint_consensus_state = storage
            .get_tendermint_consensus_state(&client_id, &height)
            .await;
        assert!(
            tendermint_consensus_state.is_ok(),
            "error: {:?}",
            tendermint_consensus_state.unwrap_err()
        );
        let tendermint_consensus_state = tendermint_consensus_state.unwrap();

        assert!(tendermint_consensus_state.is_some());
        assert_eq!(tendermint_consensus_state.unwrap(), consensus_state);
    }

    #[tokio::test]
    async fn test_connection() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let connection_id: ConnectionId = "connection-1".parse().unwrap();
        let connection_end = ConnectionEnd {
            client_id: "07-tendermint-1".to_owned(),
            versions: vec![],
            state: 0,
            counterparty: None,
            delay_period: 0,
        };

        // Add connection for a chain
        assert!(storage
            .add_connection(&connection_id, &connection_end)
            .await
            .is_ok());

        // Should not return connection for invalid connection id
        let connection = storage
            .get_connection(&"connection-2".parse().unwrap())
            .await;
        assert!(connection.is_ok(), "error: {:?}", connection.unwrap_err());
        let connection = connection.unwrap();

        assert!(connection.is_none());

        // Should return connection for valid connection id
        let connection = storage.get_connection(&connection_id).await;
        assert!(connection.is_ok(), "error: {:?}", connection.unwrap_err());
        let connection = connection.unwrap();

        assert!(connection.is_some());
        let mut connection = connection.unwrap();

        assert_eq!(connection, connection_end);

        // Update connection
        connection.state += 1;
        connection.delay_period += 1;

        assert!(storage
            .update_connection(&connection_id, &connection)
            .await
            .is_ok());

        // Should return updated connection for valid connection id
        let updated_connection = storage.get_connection(&connection_id).await;
        assert!(
            updated_connection.is_ok(),
            "error: {:?}",
            updated_connection.unwrap_err()
        );
        let updated_connection = updated_connection.unwrap();

        assert!(updated_connection.is_some());
        let updated_connection = updated_connection.unwrap();

        assert_eq!(updated_connection, connection);
    }

    #[tokio::test]
    async fn test_channel() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let port_id: PortId = "transfer".parse().unwrap();
        let channel_id: ChannelId = "channel-1".parse().unwrap();
        let channel_end = Channel {
            state: 0,
            ordering: 0,
            counterparty: None,
            connection_hops: vec![],
            version: "0".to_owned(),
        };

        // Add channel for a channel id and port id
        assert!(storage
            .add_channel(&port_id, &channel_id, &channel_end)
            .await
            .is_ok());

        // Should not return channel for invalid channel id
        let channel = storage
            .get_channel(&port_id, &"channel-2".parse().unwrap())
            .await;
        assert!(channel.is_ok(), "error: {:?}", channel.unwrap_err());
        let channel = channel.unwrap();

        assert!(channel.is_none());

        // Should not return channel for invalid port id
        let channel = storage
            .get_channel(&"port-2".parse().unwrap(), &channel_id)
            .await;
        assert!(channel.is_ok(), "error: {:?}", channel.unwrap_err());
        let channel = channel.unwrap();

        assert!(channel.is_none());

        // Should return channel for valid channel id and port id
        let channel = storage.get_channel(&port_id, &channel_id).await;
        assert!(channel.is_ok(), "error: {:?}", channel.unwrap_err());
        let channel = channel.unwrap();

        assert!(channel.is_some());
        let mut channel = channel.unwrap();

        assert_eq!(channel, channel_end);

        // Update channel
        channel.state += 1;
        channel.ordering += 1;

        assert!(storage
            .update_channel(&port_id, &channel_id, &channel)
            .await
            .is_ok());

        // Should return updated channel for valid channel id and port id
        let updated_channel = storage.get_channel(&port_id, &channel_id).await;
        assert!(
            updated_channel.is_ok(),
            "error: {:?}",
            updated_channel.unwrap_err()
        );
        let updated_channel = updated_channel.unwrap();

        assert!(updated_channel.is_some());
        let updated_channel = updated_channel.unwrap();

        assert_eq!(updated_channel, channel);
    }

    #[tokio::test]
    async fn test_ica_address() {
        let storage = SqlDbStorage::new(URI.to_owned()).await.unwrap();

        let connection_id: ConnectionId = "connection-1".parse().unwrap();
        let port_id: PortId = "transfer".parse().unwrap();
        let address_1 = "ica-address-1".to_owned();
        let address_2 = "ica-address-2".to_owned();

        // Add ica address for a connection id and port id
        assert!(storage
            .add_ica_address(&connection_id, &port_id, &address_1)
            .await
            .is_ok());

        // Should not return ica address for invalid connection id
        let ica_address = storage
            .get_ica_address(&"connection-2".parse().unwrap(), &port_id)
            .await;
        assert!(ica_address.is_ok(), "error: {:?}", ica_address.unwrap_err());
        let ica_address = ica_address.unwrap();

        assert!(ica_address.is_none());

        // Should not return ica address for invalid port id
        let ica_address = storage
            .get_ica_address(&connection_id, &"port-2".parse().unwrap())
            .await;
        assert!(ica_address.is_ok(), "error: {:?}", ica_address.unwrap_err());
        let ica_address = ica_address.unwrap();

        assert!(ica_address.is_none());

        // Should return ica address for valid connection id and port id
        let ica_address = storage.get_ica_address(&connection_id, &port_id).await;
        assert!(ica_address.is_ok(), "error: {:?}", ica_address.unwrap_err());
        let ica_address = ica_address.unwrap();

        assert!(ica_address.is_some());
        let ica_address = ica_address.unwrap();

        assert_eq!(ica_address, address_1);

        // Update ica address
        assert!(storage
            .update_ica_address(&connection_id, &port_id, &address_2)
            .await
            .is_ok());

        // Should return updated ica address for valid channel id and port id
        let updated_ica_address = storage.get_ica_address(&connection_id, &port_id).await;
        assert!(
            updated_ica_address.is_ok(),
            "error: {:?}",
            updated_ica_address.unwrap_err()
        );
        let updated_ica_address = updated_ica_address.unwrap();

        assert!(updated_ica_address.is_some());
        let updated_ica_address = updated_ica_address.unwrap();

        assert_eq!(updated_ica_address, address_2);
    }
}
