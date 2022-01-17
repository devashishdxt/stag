use anyhow::Result;
use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::core::{
        channel::v1::{
            Channel, Counterparty as ChannelCounterparty, MsgChannelOpenAck, MsgChannelOpenInit,
            Order as ChannelOrder, State as ChannelState,
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
            ics24_host::identifier::{ChannelId, ConnectionId},
        },
    },
};

pub async fn msg_channel_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    solo_machine_connection_id: &ConnectionId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let message = MsgChannelOpenInit {
        port_id: chain_state.config.port_id.to_string(),
        channel: Some(Channel {
            state: ChannelState::Init.into(),
            ordering: ChannelOrder::Unordered.into(),
            counterparty: Some(ChannelCounterparty {
                port_id: chain_state.config.port_id.to_string(),
                channel_id: "".to_string(),
            }),
            connection_hops: vec![solo_machine_connection_id.to_string()],
            version: "ics20-1".to_string(),
        }),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

pub async fn msg_channel_open_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_channel_id: &ChannelId,
    tendermint_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_try =
        get_channel_proof(context, chain_state, tendermint_channel_id, request_id).await?;

    chain_state.sequence += 1;

    let message = MsgChannelOpenAck {
        port_id: chain_state.config.port_id.to_string(),
        channel_id: solo_machine_channel_id.to_string(),
        counterparty_channel_id: tendermint_channel_id.to_string(),
        counterparty_version: "ics20-1".to_string(),
        proof_height: Some(proof_height),
        proof_try,
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
