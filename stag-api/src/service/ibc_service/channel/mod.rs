/// ICA (Interchain Account) functions
pub mod ica;
/// IBC transfer functions
pub mod transfer;

use anyhow::{Context, Result};
use cosmos_sdk_proto::ibc::core::channel::v1::State as ChannelState;

use crate::{
    event::{Event, EventHandler},
    signer::Signer,
    stag::StagContext,
    storage::{Storage, Transaction},
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        chain_state::ChainState,
        ics::core::ics24_host::identifier::{ChannelId, PortId},
    },
};

use super::common::ensure_response_success;

/// Closes an existing channel with given port id
pub async fn close_channel<C>(
    context: &C,
    chain_state: &mut ChainState,
    port_id: &PortId,
    memo: String,
    request_id: Option<&str>,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
    C::RpcClient: TendermintClient,
{
    let channel_details = chain_state.get_channel_details(port_id)?;

    close_channel_init(
        context,
        &channel_details.solo_machine_port_id,
        &channel_details.solo_machine_channel_id,
    )
    .await?;

    context
        .handle_event(Event::CloseChannelOnTendermint {
            chain_id: chain_state.id.clone(),
            port_id: channel_details.solo_machine_port_id.clone(),
        })
        .await?;

    channel_close_confirm(
        context,
        chain_state,
        &channel_details.tendermint_port_id,
        &channel_details.tendermint_channel_id,
        &channel_details.solo_machine_port_id,
        &channel_details.solo_machine_channel_id,
        memo,
        request_id,
    )
    .await?;

    context
        .handle_event(Event::CloseChannelOnTendermint {
            chain_id: chain_state.id.clone(),
            port_id: channel_details.tendermint_port_id.clone(),
        })
        .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn channel_close_confirm<C>(
    context: &C,
    chain_state: &mut ChainState,
    port_id: &PortId,
    channel_id: &ChannelId,
    counterparty_port_id: &PortId,
    counterparty_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_close_confirm(
        context,
        chain_state,
        port_id,
        channel_id,
        counterparty_port_id,
        counterparty_channel_id,
        memo,
        request_id,
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response).map(|_| ())
}

async fn close_channel_init<C>(context: &C, port_id: &PortId, channel_id: &ChannelId) -> Result<()>
where
    C: StagContext,
    C::Storage: Transaction,
{
    let mut channel = context
        .storage()
        .get_channel(port_id, channel_id)
        .await?
        .context("channel state not found")?;

    channel.set_state(ChannelState::Closed);

    context
        .storage()
        .update_channel(port_id, channel_id, &channel)
        .await?;

    Ok(())
}
