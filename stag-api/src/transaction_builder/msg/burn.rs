use anyhow::{anyhow, ensure, Result};
use cosmos_sdk_proto::{
    cosmos::{base::v1beta1::Coin, tx::v1beta1::TxRaw},
    ibc::{
        applications::transfer::v1::MsgTransfer,
        core::{
            channel::v1::{MsgAcknowledgement, Packet},
            client::v1::Height,
        },
    },
};
use primitive_types::U256;
use serde_json::json;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    transaction_builder::{proofs::get_packet_acknowledgement_proof, tx::build},
    types::{
        chain_state::ChainState,
        ics::core::{ics02_client::height::IHeight, ics24_host::identifier::Identifier},
    },
};

pub async fn msg_token_receive<C>(
    context: &C,
    chain_state: &ChainState,
    amount: U256,
    denom: &Identifier,
    receiver: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details not found for chain with id {}",
            chain_state.id
        )
    })?;
    ensure!(
        connection_details.solo_machine_channel_id.is_some(),
        "can't find solo machine channel, channel is ready closed"
    );

    let denom = chain_state.get_ibc_denom(denom)?;

    let sender = context.signer().to_account_address(&chain_state.id).await?;

    let message = MsgTransfer {
        source_port: chain_state.config.port_id.to_string(),
        source_channel: connection_details
            .solo_machine_channel_id
            .as_ref()
            .unwrap()
            .to_string(),
        token: Some(Coin {
            amount: amount.to_string(),
            denom,
        }),
        sender,
        receiver,
        timeout_height: Some(Height::new(0, u64::from(chain_state.sequence) + 1)),
        timeout_timestamp: 0,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

pub async fn msg_token_receive_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    packet: Packet,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let proof_height = Height::new(0, chain_state.sequence.into());
    let acknowledgement = serde_json::to_vec(&json!({ "result": [1] }))?;

    let proof_acked = get_packet_acknowledgement_proof(
        context,
        chain_state,
        acknowledgement.clone(),
        packet.sequence,
        request_id,
    )
    .await?;

    chain_state.sequence += 1;

    let message = MsgAcknowledgement {
        packet: Some(packet),
        acknowledgement,
        proof_acked,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
