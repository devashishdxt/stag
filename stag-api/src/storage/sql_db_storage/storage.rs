use anyhow::{Context, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use primitive_types::U256;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Pool,
};
use tendermint::node::Id as NodeId;

use crate::{
    storage::{Storage, TransactionProvider},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{
            ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
        },
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
        address: &str,
        denom: &Identifier,
        amount: &U256,
        operation_type: OperationType,
        transaction_hash: &str,
    ) -> Result<()> {
        executor::add_operation(
            &self.pool,
            request_id,
            chain_id,
            address,
            denom,
            amount,
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

    async fn delete(self) -> Result<()> {
        self.pool.close().await;

        Db::drop_database(&self.uri)
            .await
            .context("unable to drop database")
    }
}
