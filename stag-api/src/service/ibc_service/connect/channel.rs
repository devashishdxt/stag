use anyhow::{anyhow, Result};
use cosmos_sdk_proto::ibc::core::channel::v1::{
    Channel, Counterparty as ChannelCounterparty, Order as ChannelOrder, State as ChannelState,
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
        ics::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId},
    },
};

pub async fn open_channel<C>(
    context: &C,
    chain_state: &mut ChainState,
    request_id: Option<&str>,
    memo: String,
    solo_machine_connection_id: &ConnectionId,
    tendermint_connection_id: &ConnectionId,
) -> Result<(ChannelId, ChannelId)>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
    C::RpcClient: TendermintClient,
{
    let solo_machine_channel_id = channel_open_init(
        context,
        chain_state,
        solo_machine_connection_id,
        memo.clone(),
        request_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnTendermint {
            channel_id: solo_machine_channel_id.clone(),
        })
        .await?;

    let tendermint_channel_id = channel_open_try(
        context,
        &chain_state.config.port_id,
        &solo_machine_channel_id,
        tendermint_connection_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnSoloMachine {
            channel_id: tendermint_channel_id.clone(),
        })
        .await?;

    channel_open_ack(
        context,
        chain_state,
        &solo_machine_channel_id,
        &tendermint_channel_id,
        memo,
        request_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedChannelOnTendermint {
            channel_id: solo_machine_channel_id.clone(),
        })
        .await?;

    channel_open_confirm(context, &chain_state.config.port_id, &tendermint_channel_id).await?;

    context
        .handle_event(Event::ConfirmedChannelOnSoloMachine {
            channel_id: tendermint_channel_id.clone(),
        })
        .await?;

    Ok((solo_machine_channel_id, tendermint_channel_id))
}

async fn channel_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    solo_machine_connection_id: &ConnectionId,
    memo: String,
    request_id: Option<&str>,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_open_init(
        context,
        chain_state,
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

    extract_attribute(
        &response.deliver_tx.events,
        "channel_open_init",
        "channel_id",
    )?
    .parse()
}

async fn channel_open_try<C>(
    context: &C,
    port_id: &PortId,
    solo_machine_channel_id: &ChannelId,
    tendermint_connection_id: &ConnectionId,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Storage: Storage,
{
    let channel_id = ChannelId::generate();

    let channel = Channel {
        state: ChannelState::Tryopen.into(),
        ordering: ChannelOrder::Unordered.into(),
        counterparty: Some(ChannelCounterparty {
            port_id: port_id.to_string(),
            channel_id: solo_machine_channel_id.to_string(),
        }),
        connection_hops: vec![tendermint_connection_id.to_string()],
        version: "ics20-1".to_string(),
    };

    context
        .storage()
        .add_channel(port_id, &channel_id, &channel)
        .await?;

    Ok(channel_id)
}

async fn channel_open_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_channel_id: &ChannelId,
    tendermint_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_open_ack(
        context,
        chain_state,
        solo_machine_channel_id,
        tendermint_channel_id,
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

async fn channel_open_confirm<C>(
    context: &C,
    port_id: &PortId,
    channel_id: &ChannelId,
) -> Result<()>
where
    C: StagContext,
    C::Storage: Transaction,
{
    let mut channel = context
        .storage()
        .get_channel(port_id, channel_id)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "channel for channel id ({}) and port id ({}) not found",
                channel_id,
                port_id
            )
        })?;

    channel.set_state(ChannelState::Open);

    context
        .storage()
        .update_channel(port_id, channel_id, &channel)
        .await
}
