use anyhow::Result;
use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::core::{
        channel::v1::{
            Channel, Counterparty as ChannelCounterparty, MsgChannelCloseConfirm,
            MsgChannelCloseInit, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
            MsgChannelOpenTry, Order as ChannelOrder, State as ChannelState,
        },
        client::v1::Height,
    },
};

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::Storage,
    transaction_builder::{proofs::get_channel_proof, tx::build},
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::identifier::{ChannelId, ConnectionId, PortId},
        },
    },
};

#[allow(clippy::too_many_arguments)]
/// Creates a message for opening (init) a channel on IBC enabled chain
pub async fn msg_channel_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    connection_id: &ConnectionId,
    port_id: &PortId,
    counterparty_port_id: &PortId,
    ordering: ChannelOrder,
    version: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let message = MsgChannelOpenInit {
        port_id: port_id.to_string(),
        channel: Some(Channel {
            state: ChannelState::Init.into(),
            ordering: ordering.into(),
            counterparty: Some(ChannelCounterparty {
                port_id: counterparty_port_id.to_string(),
                channel_id: "".to_string(),
            }),
            connection_hops: vec![connection_id.to_string()],
            version,
        }),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

#[allow(clippy::too_many_arguments)]
/// Creates a message for opening (init) a channel on IBC enabled chain
pub async fn msg_channel_open_try<C>(
    context: &C,
    chain_state: &mut ChainState,
    connection_id: &ConnectionId,
    port_id: &PortId,
    counterparty_channel_id: &ChannelId,
    counterparty_port_id: &PortId,
    counterparty_version: String,
    ordering: ChannelOrder,
    version: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_init = get_channel_proof(
        context,
        chain_state,
        counterparty_channel_id,
        counterparty_port_id,
        request_id,
    )
    .await?;

    let message = MsgChannelOpenTry {
        port_id: port_id.to_string(),
        previous_channel_id: "".to_owned(),
        channel: Some(Channel {
            state: ChannelState::Tryopen.into(),
            ordering: ordering.into(),
            counterparty: Some(ChannelCounterparty {
                port_id: counterparty_port_id.to_string(),
                channel_id: counterparty_channel_id.to_string(),
            }),
            connection_hops: vec![connection_id.to_string()],
            version,
        }),
        counterparty_version,
        proof_init,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

#[allow(clippy::too_many_arguments)]
/// Creates a message for acknowledging a channel on IBC enabled chain
pub async fn msg_channel_open_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    channel_id: &ChannelId,
    port_id: &PortId,
    counterparty_channel_id: &ChannelId,
    counterparty_port_id: &PortId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_try = get_channel_proof(
        context,
        chain_state,
        counterparty_channel_id,
        counterparty_port_id,
        request_id,
    )
    .await?;

    let message = MsgChannelOpenAck {
        port_id: port_id.to_string(),
        channel_id: channel_id.to_string(),
        counterparty_channel_id: counterparty_channel_id.to_string(),
        counterparty_version: "ics20-1".to_string(),
        proof_height: Some(proof_height),
        proof_try,
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

#[allow(clippy::too_many_arguments)]
/// Creates a message for confirming a channel on IBC enabled chain
pub async fn msg_channel_open_confirm<C>(
    context: &C,
    chain_state: &mut ChainState,
    port_id: &PortId,
    channel_id: &ChannelId,
    counterparty_port_id: &PortId,
    counterparty_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_ack = get_channel_proof(
        context,
        chain_state,
        counterparty_channel_id,
        counterparty_port_id,
        request_id,
    )
    .await?;

    let message = MsgChannelOpenConfirm {
        port_id: port_id.to_string(),
        channel_id: channel_id.to_string(),
        proof_ack,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

/// Creates a message for initiating closing of channel
pub async fn msg_channel_close_init<C>(
    context: &C,
    chain_state: &ChainState,
    port_id: &PortId,
    channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let message = MsgChannelCloseInit {
        port_id: port_id.to_string(),
        channel_id: channel_id.to_string(),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

/// Creates a message for confirming closing on channel
#[allow(clippy::too_many_arguments)]
pub async fn msg_channel_close_confirm<C>(
    context: &C,
    chain_state: &mut ChainState,
    port_id: &PortId,
    channel_id: &ChannelId,
    counterparty_port_id: &PortId,
    counterparty_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_init = get_channel_proof(
        context,
        chain_state,
        counterparty_channel_id,
        counterparty_port_id,
        request_id,
    )
    .await?;

    let message = MsgChannelCloseConfirm {
        port_id: port_id.to_string(),
        channel_id: channel_id.to_string(),
        proof_init,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
