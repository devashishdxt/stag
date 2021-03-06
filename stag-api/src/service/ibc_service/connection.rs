use anyhow::{anyhow, Result};
use cosmos_sdk_proto::ibc::core::{
    commitment::v1::MerklePrefix,
    connection::v1::{
        ConnectionEnd, Counterparty as ConnectionCounterparty, State as ConnectionState,
        Version as ConnectionVersion,
    },
};

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{ensure_response_success, extract_attribute},
    signer::Signer,
    stag::StagContext,
    storage::{Storage, Transaction},
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        chain_state::ChainState,
        ics::core::ics24_host::identifier::{ClientId, ConnectionId},
    },
};

pub async fn establish_connection<C>(
    context: &C,
    chain_state: &mut ChainState,
    request_id: Option<&str>,
    memo: String,
    solo_machine_client_id: &ClientId,
    tendermint_client_id: &ClientId,
) -> Result<(ConnectionId, ConnectionId)>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
    C::RpcClient: TendermintClient,
{
    let tendermint_connection_id = connection_open_init(
        context,
        chain_state,
        request_id,
        memo.clone(),
        solo_machine_client_id,
        tendermint_client_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedConnectionOnTendermint {
            connection_id: tendermint_connection_id.clone(),
        })
        .await?;

    let solo_machine_connection_id = connection_open_try(
        context,
        tendermint_client_id,
        solo_machine_client_id,
        &tendermint_connection_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedConnectionOnSoloMachine {
            connection_id: solo_machine_connection_id.clone(),
        })
        .await?;

    connection_open_ack(
        context,
        chain_state,
        request_id,
        memo,
        solo_machine_client_id,
        &solo_machine_connection_id,
        &tendermint_connection_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedConnectionOnTendermint {
            connection_id: tendermint_connection_id.clone(),
        })
        .await?;

    connection_open_confirm(context, &solo_machine_connection_id).await?;

    context
        .handle_event(Event::ConfirmedConnectionOnSoloMachine {
            connection_id: solo_machine_connection_id.clone(),
        })
        .await?;

    Ok((solo_machine_connection_id, tendermint_connection_id))
}

async fn connection_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    request_id: Option<&str>,
    memo: String,
    solo_machine_client_id: &ClientId,
    tendermint_client_id: &ClientId,
) -> Result<ConnectionId>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_connection_open_init(
        context,
        chain_state,
        tendermint_client_id,
        solo_machine_client_id,
        memo,
        request_id,
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    extract_attribute(
        &response.deliver_tx.events,
        "connection_open_init",
        "connection_id",
    )?
    .parse()
}

async fn connection_open_try<C>(
    context: &C,
    tendermint_client_id: &ClientId,
    solo_machine_client_id: &ClientId,
    tendermint_connection_id: &ConnectionId,
) -> Result<ConnectionId>
where
    C: StagContext,
    C::Storage: Storage,
{
    let connection_id = ConnectionId::generate();

    let connection = ConnectionEnd {
        client_id: solo_machine_client_id.to_string(),
        counterparty: Some(ConnectionCounterparty {
            client_id: tendermint_client_id.to_string(),
            connection_id: tendermint_connection_id.to_string(),
            prefix: Some(MerklePrefix {
                key_prefix: "ibc".as_bytes().to_vec(),
            }),
        }),
        versions: vec![ConnectionVersion {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        }],
        state: ConnectionState::Tryopen.into(),
        delay_period: 0,
    };

    context
        .storage()
        .add_connection(&connection_id, &connection)
        .await?;

    Ok(connection_id)
}

async fn connection_open_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    request_id: Option<&str>,
    memo: String,
    solo_machine_client_id: &ClientId,
    solo_machine_connection_id: &ConnectionId,
    tendermint_connection_id: &ConnectionId,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_connection_open_ack(
        context,
        chain_state,
        tendermint_connection_id,
        solo_machine_client_id,
        solo_machine_connection_id,
        memo,
        request_id,
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    Ok(())
}

async fn connection_open_confirm<C>(context: &C, connection_id: &ConnectionId) -> Result<()>
where
    C: StagContext,
    C::Storage: Transaction,
{
    let mut connection = context
        .storage()
        .get_connection(connection_id)
        .await?
        .ok_or_else(|| anyhow!("connection for connection id ({}) not found", connection_id))?;

    connection.set_state(ConnectionState::Open);

    context
        .storage()
        .update_connection(connection_id, &connection)
        .await
}
