use anyhow::{anyhow, Result};
use cosmos_sdk_proto::ibc::core::channel::v1::{
    Channel, Counterparty as ChannelCounterparty, Order as ChannelOrder, State as ChannelState,
};
use serde_json::json;

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{ensure_response_success, extract_attribute},
    signer::Signer,
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        chain_state::{ChainState, ChannelDetails},
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
) -> Result<ChannelDetails>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let solo_machine_port_id = PortId::ica_controller(context.signer(), &chain_state.id).await?;
    let tendermint_port_id = PortId::ica_host();

    let solo_machine_version = serde_json::to_string(&json!({
       "version": "ics27-1",
       "controller_connection_id": solo_machine_connection_id,
       "host_connection_id": tendermint_connection_id,
       "address": "",
       "encoding": "proto3",
       "tx_type": "sdk_multi_msg",
    }))?;

    let solo_machine_channel_id = channel_open_init(
        context,
        &solo_machine_port_id,
        tendermint_connection_id,
        &tendermint_port_id,
        solo_machine_version.clone(),
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnSoloMachine {
            channel_id: solo_machine_channel_id.clone(),
            port_id: solo_machine_port_id.clone(),
        })
        .await?;

    let tendermint_channel_id = channel_open_try(
        context,
        chain_state,
        &solo_machine_channel_id,
        &solo_machine_port_id,
        &tendermint_port_id,
        solo_machine_version,
        memo.clone(),
        request_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnTendermint {
            channel_id: tendermint_channel_id.clone(),
            port_id: tendermint_port_id.clone(),
        })
        .await?;

    channel_open_ack(
        context,
        &solo_machine_port_id,
        &solo_machine_channel_id,
        &tendermint_channel_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedChannelOnSoloMachine {
            channel_id: solo_machine_channel_id.clone(),
            port_id: solo_machine_port_id.clone(),
        })
        .await?;

    channel_open_confirm(
        context,
        chain_state,
        &solo_machine_port_id,
        &solo_machine_channel_id,
        &tendermint_port_id,
        &tendermint_channel_id,
        memo,
        request_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedChannelOnTendermint {
            channel_id: tendermint_channel_id.clone(),
            port_id: tendermint_port_id.clone(),
        })
        .await?;

    Ok(ChannelDetails {
        packet_sequence: 1,
        solo_machine_port_id,
        tendermint_port_id,
        solo_machine_channel_id,
        tendermint_channel_id,
    })
}

async fn channel_open_init<C>(
    context: &C,
    solo_machine_port_id: &PortId,
    tendermint_connection_id: &ConnectionId,
    tendermint_port_id: &PortId,
    solo_machine_version: String,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Storage: Storage,
{
    let channel_id = ChannelId::generate();

    let channel = Channel {
        state: ChannelState::Init.into(),
        ordering: ChannelOrder::Ordered.into(),
        counterparty: Some(ChannelCounterparty {
            port_id: tendermint_port_id.to_string(),
            channel_id: "".to_string(),
        }),
        connection_hops: vec![tendermint_connection_id.to_string()],
        version: solo_machine_version,
    };

    context
        .storage()
        .add_channel(solo_machine_port_id, &channel_id, &channel)
        .await?;

    Ok(channel_id)
}

#[allow(clippy::too_many_arguments)]
async fn channel_open_try<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_channel_id: &ChannelId,
    solo_machine_port_id: &PortId,
    tendermint_port_id: &PortId,
    solo_machine_version: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_open_try(
        context,
        chain_state,
        tendermint_port_id,
        solo_machine_channel_id,
        solo_machine_port_id,
        solo_machine_version,
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
        "channel_open_try",
        "channel_id",
    )?
    .parse()
}

async fn channel_open_ack<C>(
    context: &C,
    solo_machine_port_id: &PortId,
    solo_machine_channel_id: &ChannelId,
    tendermint_channel_id: &ChannelId,
) -> Result<()>
where
    C: StagContext,
    C::Storage: Storage,
{
    let mut channel = context
        .storage()
        .get_channel(solo_machine_port_id, solo_machine_channel_id)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "channel for channel id ({}) and port id ({}) not found",
                solo_machine_channel_id,
                solo_machine_port_id
            )
        })?;

    channel.set_state(ChannelState::Open);

    let counterparty = channel.counterparty.as_mut().ok_or_else(|| {
        anyhow!(
            "counterparty not set for channel id ({}) and port id ({})",
            solo_machine_channel_id,
            solo_machine_port_id
        )
    })?;

    counterparty.channel_id = tendermint_channel_id.to_string();

    context
        .storage()
        .update_channel(solo_machine_port_id, solo_machine_channel_id, &channel)
        .await
}

#[allow(clippy::too_many_arguments)]
async fn channel_open_confirm<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    solo_machine_channel_id: &ChannelId,
    tendermint_port_id: &PortId,
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
    let msg = transaction_builder::msg_channel_open_confirm(
        context,
        chain_state,
        tendermint_port_id,
        tendermint_channel_id,
        solo_machine_port_id,
        solo_machine_channel_id,
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
