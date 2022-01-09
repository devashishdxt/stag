use anyhow::{anyhow, ensure, Result};
use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::core::{
        channel::v1::{MsgRecvPacket, Packet},
        client::v1::Height,
    },
};
use primitive_types::U256;
use serde::Serialize;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    tendermint::TendermintClient,
    transaction_builder::{proofs::get_packet_commitment_proof, tx::build},
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::identifier::{ChainId, Identifier},
        },
    },
};

#[derive(Debug, Serialize)]
struct TokenTransferPacketData {
    pub denom: String,
    // Ideally `amount` should be `U256` but `ibc-go` uses `protojson` which encodes `uint256` into `string`. So, using
    // `String` here to keep consistent wire format.
    pub amount: String,
    pub sender: String,
    pub receiver: String,
}

/// Creates and signs a `MsgRecvPacket` transaction.
pub async fn msg_token_send<C>(
    context: &C,
    chain_state: &mut ChainState,
    amount: U256,
    denom: &Identifier,
    receiver: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details not found for chain with id {}",
            chain_state.id
        )
    })?;
    ensure!(
        connection_details.solo_machine_channel_id.is_some(),
        "can't find solo machine channel, channel is already closed"
    );
    ensure!(
        connection_details.tendermint_channel_id.is_some(),
        "can't find tendermint channel, channel is already closed"
    );

    let sender = context.signer().to_account_address(&chain_state.id)?;

    let packet_data = TokenTransferPacketData {
        denom: denom.to_string(),
        amount: amount.to_string(),
        sender: sender.clone(),
        receiver,
    };

    let packet = Packet {
        sequence: chain_state.packet_sequence.into(),
        source_port: chain_state.config.port_id.to_string(),
        source_channel: connection_details
            .tendermint_channel_id
            .as_ref()
            .unwrap()
            .to_string(),
        destination_port: chain_state.config.port_id.to_string(),
        destination_channel: connection_details
            .solo_machine_channel_id
            .as_ref()
            .unwrap()
            .to_string(),
        data: serde_json::to_vec(&packet_data)?,
        timeout_height: Some(
            get_latest_height(context, chain_state)
                .await?
                .checked_add(chain_state.config.packet_timeout_height_offset)
                .ok_or_else(|| anyhow!("height addition overflow"))?,
        ),
        timeout_timestamp: 0,
    };

    let proof_commitment =
        get_packet_commitment_proof(context, chain_state, &packet, request_id).await?;

    let proof_height = Height::new(0, chain_state.sequence.into());

    chain_state.sequence += 1;
    chain_state.packet_sequence += 1;

    let message = MsgRecvPacket {
        packet: Some(packet),
        proof_commitment,
        proof_height: Some(proof_height),
        signer: sender,
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
