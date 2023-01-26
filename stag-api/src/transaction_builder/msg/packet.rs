use anyhow::{anyhow, ensure, Context, Result};

use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::core::{
        channel::v1::{MsgAcknowledgement, MsgRecvPacket, Packet},
        client::v1::Height,
    },
};
use serde_json::json;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    tendermint::TendermintClient,
    transaction_builder::{
        proofs::{get_packet_acknowledgement_proof, get_packet_commitment_proof},
        tx::build,
    },
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::identifier::{ChainId, PortId},
        },
    },
};

/// Creates a message for sending a packet on IBC enabled chain
pub async fn msg_receive_packet<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    packet_data: Vec<u8>,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let connection_details = chain_state
        .connection_details
        .as_ref()
        .ok_or_else(|| anyhow!("connection details for chain {} not found", chain_state.id))?;

    let channel_details = connection_details
        .channels
        .get(solo_machine_port_id)
        .ok_or_else(|| {
            anyhow!(
                "channel details for port {} not found",
                solo_machine_port_id
            )
        })?;

    let packet = Packet {
        sequence: channel_details.packet_sequence.into(),
        source_port: solo_machine_port_id.to_string(),
        source_channel: channel_details.solo_machine_channel_id.to_string(),
        destination_port: channel_details.tendermint_port_id.to_string(),
        destination_channel: channel_details.tendermint_channel_id.to_string(),
        data: packet_data,
        timeout_height: Some(
            get_latest_height(context, chain_state)
                .await?
                .checked_add(chain_state.config.packet_timeout_height_offset)
                .context("height addition overflow")?,
        ),
        timeout_timestamp: 0,
    };

    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_commitment = get_packet_commitment_proof(
        context,
        chain_state,
        solo_machine_port_id,
        &packet,
        request_id,
    )
    .await?;

    let connection_details = chain_state
        .connection_details
        .as_mut()
        .ok_or_else(|| anyhow!("connection details for chain {} not found", chain_state.id))?;

    let channel_details = connection_details
        .channels
        .get_mut(solo_machine_port_id)
        .ok_or_else(|| {
            anyhow!(
                "channel details for port {} not found",
                solo_machine_port_id
            )
        })?;

    channel_details.packet_sequence += 1;

    let message = MsgRecvPacket {
        packet: Some(packet),
        proof_commitment,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    let tx_raw = build(context, chain_state, &[message], memo, request_id).await?;

    Ok(tx_raw)
}

/// Creates a message for acknowledging a packet on IBC enabled chain
pub async fn msg_acknowledgement<C>(
    context: &C,
    chain_state: &mut ChainState,
    port_id: &PortId,
    packet: Packet,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let proof_height = Height::new(0, chain_state.sequence.into());
    let acknowledgement = serde_json::to_vec(&json!({ "result": [1u8] }))?;

    let proof_acked = get_packet_acknowledgement_proof(
        context,
        chain_state,
        port_id,
        acknowledgement.clone(),
        packet.sequence,
        request_id,
    )
    .await?;

    let message = MsgAcknowledgement {
        packet: Some(packet),
        acknowledgement,
        proof_acked,
        proof_height: Some(proof_height),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

async fn get_latest_height<C>(context: &C, chain_state: &ChainState) -> Result<Height>
where
    C: StagContext,
    C::RpcClient: TendermintClient,
{
    let response = context
        .rpc_client()
        .status(&chain_state.config.rpc_addr)
        .await?;

    ensure!(
        !response.sync_info.catching_up,
        "node at {} running chain {} not caught up",
        chain_state.config.rpc_addr,
        chain_state.id,
    );

    let revision_number = response
        .node_info
        .network
        .as_str()
        .parse::<ChainId>()?
        .version();

    let revision_height = response.sync_info.latest_block_height.into();

    Ok(Height {
        revision_number,
        revision_height,
    })
}
