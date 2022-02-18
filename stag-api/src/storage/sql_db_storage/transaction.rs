use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use primitive_types::U256;
use tendermint::node::Id as NodeId;
use tokio::sync::Mutex;

use crate::{
    storage::{Storage, Transaction},
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{
            ChainId, ChannelId, ClientId, ConnectionId, Identifier, PortId,
        },
        operation::{Operation, OperationType},
    },
};

use super::{executor, Db};

pub struct SqlDbTransaction {
    transaction: Mutex<sqlx::Transaction<'static, Db>>,
}

impl SqlDbTransaction {
    pub fn new(transaction: sqlx::Transaction<'static, Db>) -> Self {
        Self {
            transaction: Mutex::new(transaction),
        }
    }
}

#[async_trait]
impl Transaction for SqlDbTransaction {
    async fn done(self) -> Result<()> {
        self.transaction
            .into_inner()
            .commit()
            .await
            .context("unable to commit transaction")
    }
}

#[async_trait]
impl Storage for SqlDbTransaction {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_chain_state(&mut *transaction, chain_id, node_id, chain_config).await
    }

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_chain_state(&mut *transaction, chain_id).await
    }

    async fn update_chain_state(&self, chain_state: &ChainState) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::update_chain_state(&mut *transaction, chain_state).await
    }

    async fn get_all_chain_states(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainState>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_all_chain_states(&mut *transaction, limit, offset).await
    }

    async fn add_chain_key(&self, chain_id: &ChainId, public_key: &str) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_chain_key(&mut *transaction, chain_id, public_key).await
    }

    async fn get_chain_keys(
        &self,
        chain_id: &ChainId,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<ChainKey>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_chain_keys(&mut *transaction, chain_id, limit, offset).await
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
        let mut transaction = self.transaction.lock().await;

        executor::add_operation(
            &mut *transaction,
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
        let mut transaction = self.transaction.lock().await;

        executor::get_operations(&mut *transaction, chain_id, limit, offset).await
    }

    async fn add_tendermint_client_state(
        &self,
        client_id: &ClientId,
        client_state: &TendermintClientState,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_tendermint_client_state(&mut *transaction, client_id, client_state).await
    }

    async fn get_tendermint_client_state(
        &self,
        client_id: &ClientId,
    ) -> Result<Option<TendermintClientState>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_tendermint_client_state(&mut *transaction, client_id).await
    }

    async fn add_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
        consensus_state: &TendermintConsensusState,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_tendermint_consensus_state(
            &mut *transaction,
            client_id,
            height,
            consensus_state,
        )
        .await
    }

    async fn get_tendermint_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<TendermintConsensusState>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_tendermint_consensus_state(&mut *transaction, client_id, height).await
    }

    async fn add_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_connection(&mut *transaction, connection_id, connection).await
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<ConnectionEnd>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_connection(&mut *transaction, connection_id).await
    }

    async fn update_connection(
        &self,
        connection_id: &ConnectionId,
        connection: &ConnectionEnd,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::update_connection(&mut *transaction, connection_id, connection).await
    }

    async fn add_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::add_channel(&mut *transaction, port_id, channel_id, channel).await
    }

    async fn get_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Option<Channel>> {
        let mut transaction = self.transaction.lock().await;

        executor::get_channel(&mut *transaction, port_id, channel_id).await
    }

    async fn update_channel(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        channel: &Channel,
    ) -> Result<()> {
        let mut transaction = self.transaction.lock().await;

        executor::update_channel(&mut *transaction, port_id, channel_id, channel).await
    }

    async fn delete(self) -> Result<()> {
        Err(anyhow!("cannot delete storage from a transaction"))
    }
}
