use std::fmt::Write;

use anyhow::{bail, ensure, Context, Result};
use chrono::Utc;
use cosmos_sdk_proto::ibc::{
    core::{channel::v1::Channel, client::v1::Height, connection::v1::ConnectionEnd},
    lightclients::tendermint::v1::{
        ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
    },
};
use prost::Message;
use sqlx::{types::Json, Executor};
use tendermint::node::Id as NodeId;

use crate::types::{
    chain_state::{ChainConfig, ChainKey, ChainState},
    ibc_data::IbcData,
    ics::core::ics24_host::{
        identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
        path::{
            ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath,
            InterchainAccountAddressPath,
        },
    },
    operation::{Operation, OperationType},
    proto_util::proto_encode,
};

use super::{Db, DbRow};

pub async fn add_chain_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_id: ChainId,
    node_id: NodeId,
    chain_config: ChainConfig,
) -> Result<()> {
    let rows_affected =
        sqlx::query("INSERT INTO chain_states (id, node_id, config) VALUES ($1, $2, $3)")
            .bind(chain_id.to_string())
            .bind(node_id.to_string())
            .bind(Json(chain_config))
            .execute(executor)
            .await
            .context("unable to add chain details in database")?
            .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when adding new chain details"
    );

    Ok(())
}

pub async fn get_chain_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_id: &ChainId,
) -> Result<Option<ChainState>> {
    let row: Option<DbRow> = sqlx::query("SELECT * FROM chain_states WHERE id = $1")
        .bind(chain_id.to_string())
        .fetch_optional(executor)
        .await
        .context("unable to get chain state from database")?;

    row.map(TryFrom::try_from).transpose()
}

pub async fn update_chain_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_state: &ChainState,
) -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "postgres-storage")] {
            let sequence: i64 = chain_state.sequence.into();
        } else if #[cfg(feature = "sqlite-storage")] {
            let sequence: u32 = chain_state.sequence;
        }
    }

    let rows_affected =
        sqlx::query("UPDATE chain_states SET node_id = $1, config = $2, consensus_timestamp = $3, sequence = $4, connection_details = $5, updated_at = $6 WHERE id = $7")
            .bind(chain_state.node_id.to_string())
            .bind(Json(&chain_state.config))
            .bind(chain_state.consensus_timestamp)
            .bind(sequence)
            .bind(chain_state.connection_details.as_ref().map(Json))
            .bind(Utc::now())
            .bind(chain_state.id.to_string())
            .execute(executor)
            .await
            .context("unable to add chain details in database")?
            .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when adding new chain details"
    );

    Ok(())
}

pub async fn get_all_chain_states<'e>(
    executor: impl Executor<'e, Database = Db>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ChainState>> {
    let mut query = "SELECT * FROM chain_states".to_owned();

    push_limit_offset(&mut query, limit, offset)?;

    let raw: Vec<DbRow> = sqlx::query(&query)
        .fetch_all(executor)
        .await
        .context("unable to query account operations from database")?;

    raw.into_iter().map(TryInto::try_into).collect()
}

pub async fn add_chain_key<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_id: &ChainId,
    public_key: &str,
) -> Result<()> {
    let rows_affected =
        sqlx::query("INSERT INTO chain_keys (chain_id, public_key) VALUES ($1, $2)")
            .bind(chain_id.to_string())
            .bind(public_key)
            .execute(executor)
            .await
            .context("unable to add new chain key")?
            .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when adding new chain key"
    );

    Ok(())
}

pub async fn get_chain_keys<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_id: &ChainId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ChainKey>> {
    let mut query = "SELECT * FROM chain_keys WHERE chain_id = $1 ORDER BY id DESC".to_owned();

    push_limit_offset(&mut query, limit, offset)?;

    let raw: Vec<DbRow> = sqlx::query(&query)
        .bind(chain_id.to_string())
        .fetch_all(executor)
        .await
        .context("unable to query chain keys from database")?;

    raw.into_iter().map(TryFrom::try_from).collect()
}

pub async fn get_operations<'e>(
    executor: impl Executor<'e, Database = Db>,
    chain_id: &str,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Operation>> {
    let mut query = "SELECT * FROM operations WHERE chain_id = $1 ORDER BY id DESC".to_owned();

    push_limit_offset(&mut query, limit, offset)?;

    let raw: Vec<DbRow> = sqlx::query(&query)
        .bind(chain_id)
        .fetch_all(executor)
        .await
        .context("unable to query account operations from database")?;

    raw.into_iter().map(TryFrom::try_from).collect()
}

#[allow(clippy::too_many_arguments)]
pub async fn add_operation<'e>(
    executor: impl Executor<'e, Database = Db>,
    request_id: Option<&str>,
    chain_id: &ChainId,
    port_id: &PortId,
    operation_type: &OperationType,
    transaction_hash: &str,
) -> Result<()> {
    let rows_affected = sqlx::query(
            "INSERT INTO operations (request_id, chain_id, port_id, operation_type, transaction_hash) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(request_id)
        .bind(chain_id.to_string())
        .bind(port_id.to_string())
        .bind(Json(&operation_type))
        .bind(transaction_hash)
        .execute(executor)
        .await
        .context("unable to add new account operation to database")?
        .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when adding a new account operation"
    );

    Ok(())
}

pub async fn add_tendermint_client_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    client_id: &ClientId,
    client_state: &TendermintClientState,
) -> Result<()> {
    let path: String = ClientStatePath::new(client_id).into();
    let data = proto_encode(client_state)?;

    add_ibc_data(executor, path, data).await
}

pub async fn get_tendermint_client_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    client_id: &ClientId,
) -> Result<Option<TendermintClientState>> {
    let path: String = ClientStatePath::new(client_id).into();
    get_ibc_data(executor, &path).await
}

pub async fn add_tendermint_consensus_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    client_id: &ClientId,
    height: &Height,
    consensus_state: &TendermintConsensusState,
) -> Result<()> {
    let path: String = ConsensusStatePath::new(client_id, height).into();
    let data = proto_encode(consensus_state)?;

    add_ibc_data(executor, path, data).await
}

pub async fn get_tendermint_consensus_state<'e>(
    executor: impl Executor<'e, Database = Db>,
    client_id: &ClientId,
    height: &Height,
) -> Result<Option<TendermintConsensusState>> {
    let path: String = ConsensusStatePath::new(client_id, height).into();
    get_ibc_data(executor, &path).await
}

pub async fn add_connection<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
    connection: &ConnectionEnd,
) -> Result<()> {
    let path: String = ConnectionPath::new(connection_id).into();
    let data = proto_encode(connection)?;

    add_ibc_data(executor, path, data).await
}

pub async fn get_connection<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
) -> Result<Option<ConnectionEnd>> {
    let path: String = ConnectionPath::new(connection_id).into();
    get_ibc_data(executor, &path).await
}

pub async fn update_connection<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
    connection: &ConnectionEnd,
) -> Result<()> {
    let path: String = ConnectionPath::new(connection_id).into();
    let data = proto_encode(connection)?;

    update_ibc_data(executor, path, data).await
}

pub async fn add_channel<'e>(
    executor: impl Executor<'e, Database = Db>,
    port_id: &PortId,
    channel_id: &ChannelId,
    channel: &Channel,
) -> Result<()> {
    let path: String = ChannelPath::new(port_id, channel_id).into();
    let data = proto_encode(channel)?;

    add_ibc_data(executor, path, data).await
}

pub async fn get_channel<'e>(
    executor: impl Executor<'e, Database = Db>,
    port_id: &PortId,
    channel_id: &ChannelId,
) -> Result<Option<Channel>> {
    let path: String = ChannelPath::new(port_id, channel_id).into();
    get_ibc_data(executor, &path).await
}

pub async fn update_channel<'e>(
    executor: impl Executor<'e, Database = Db>,
    port_id: &PortId,
    channel_id: &ChannelId,
    channel: &Channel,
) -> Result<()> {
    let path: String = ChannelPath::new(port_id, channel_id).into();
    let data = proto_encode(channel)?;

    update_ibc_data(executor, path, data).await
}

pub async fn add_ica_address<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
    port_id: &PortId,
    address: &str,
) -> Result<()> {
    let path: String = InterchainAccountAddressPath::new(connection_id, port_id).into();
    let data = proto_encode(&address.to_owned())?;

    add_ibc_data(executor, path, data).await
}

pub async fn get_ica_address<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
    port_id: &PortId,
) -> Result<Option<String>> {
    let path: String = InterchainAccountAddressPath::new(connection_id, port_id).into();
    get_ibc_data(executor, &path).await
}

pub async fn update_ica_address<'e>(
    executor: impl Executor<'e, Database = Db>,
    connection_id: &ConnectionId,
    port_id: &PortId,
    address: &str,
) -> Result<()> {
    let path: String = InterchainAccountAddressPath::new(connection_id, port_id).into();
    let data = proto_encode(&address.to_owned())?;

    update_ibc_data(executor, path, data).await
}

async fn add_ibc_data<'e>(
    executor: impl Executor<'e, Database = Db>,
    path: String,
    data: Vec<u8>,
) -> Result<()> {
    let rows_affected = sqlx::query("INSERT INTO ibc_data (path, data) VALUES ($1, $2)")
        .bind(path)
        .bind(data)
        .execute(executor)
        .await
        .context("unable to add ibc data in database")?
        .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when adding a new ibc data"
    );

    Ok(())
}

async fn update_ibc_data<'e>(
    executor: impl Executor<'e, Database = Db>,
    path: String,
    data: Vec<u8>,
) -> Result<()> {
    let rows_affected = sqlx::query("UPDATE ibc_data SET data = $1 where path = $2")
        .bind(data)
        .bind(path)
        .execute(executor)
        .await
        .context("unable to update ibc data in database")?
        .rows_affected();

    ensure!(
        rows_affected == 1,
        "rows_affected should be equal to 1 when updating ibc data"
    );

    Ok(())
}

async fn get_ibc_data<'e, M>(
    executor: impl Executor<'e, Database = Db>,
    path: &str,
) -> Result<Option<M>>
where
    M: Message + Default,
{
    let raw: Option<DbRow> = sqlx::query("SELECT * FROM ibc_data WHERE path = $1")
        .bind(path)
        .fetch_optional(executor)
        .await
        .context("unable to query ibc data from database")?;

    raw.map(|row| -> Result<M> {
        let ibc_data = IbcData::try_from(row)?;
        M::decode(ibc_data.data.as_ref()).context("unable to decode protobuf bytes for ibc data")
    })
    .transpose()
}

fn push_limit_offset(query: &mut String, limit: Option<u32>, offset: Option<u32>) -> Result<()> {
    if let Some(limit) = limit {
        write!(query, " LIMIT {}", limit)?;
    }

    if let Some(offset) = offset {
        if limit.is_none() {
            bail!("offset cannot be set without limit");
        }

        write!(query, " OFFSET {}", offset)?;
    }

    Ok(())
}
