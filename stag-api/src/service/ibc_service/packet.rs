use std::collections::HashMap;

use anyhow::{anyhow, ensure, Context, Result};
use cosmos_sdk_proto::ibc::core::{channel::v1::Packet, client::v1::Height};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response as TxCommitResponse;

use crate::{
    service::ibc_service::common::ensure_response_success,
    signer::Signer,
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        chain_state::ChainState,
        ics::core::{ics02_client::height::IHeight, ics24_host::identifier::PortId},
    },
};

pub fn extract_packets(response: &TxCommitResponse) -> Result<Vec<Packet>> {
    let mut packets = vec![];

    for event in response.deliver_tx.events.iter() {
        if event.kind == "send_packet" {
            let mut attributes = HashMap::new();

            for tag in event.attributes.iter() {
                attributes.insert(tag.key.to_string(), tag.value.to_string());
            }

            let packet = Packet {
                sequence: attributes
                    .remove("packet_sequence")
                    .context("`packet_sequence` is missing from packet data")?
                    .parse()
                    .context("invalid `packet_sequence`")?,
                source_port: attributes
                    .remove("packet_src_port")
                    .context("`packet_src_port` is missing from packet data")?,
                source_channel: attributes
                    .remove("packet_src_channel")
                    .context("`packet_src_channel` is missing from packet data")?,
                destination_port: attributes
                    .remove("packet_dst_port")
                    .context("`packet_dst_port` is missing from packet data")?,
                destination_channel: attributes
                    .remove("packet_dst_channel")
                    .context("`packet_dst_channel` is missing from packet data")?,
                data: attributes
                    .remove("packet_data")
                    .context("`packet_data` is missing from packet data")?
                    .into_bytes(),
                timeout_height: Some(
                    Height::from_str(&attributes.remove("packet_timeout_height").ok_or_else(
                        || anyhow!("`packet_timeout_height` is missing from packet data"),
                    )?)
                    .context("invalid `packet_timeout_height`")?,
                ),
                timeout_timestamp: attributes
                    .remove("packet_timeout_timestamp")
                    .context("`packet_timeout_timestamp` is missing from packet data")?
                    .parse()
                    .context("invalid `packet_timeout_timestamp`")?,
            };

            packets.push(packet);
        }
    }

    Ok(packets)
}

pub async fn process_packets<C>(
    context: &C,
    chain_state: &ChainState,
    port_id: &PortId,
    packets: Vec<Packet>,
    memo: String,
    request_id: Option<String>,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details for chain with id {} are missing",
            chain_state.id
        )
    })?;

    let channel_details = connection_details
        .channels
        .get(port_id)
        .ok_or_else(|| anyhow!("channel details for port {} are missing", port_id))?;

    let solo_machine_channel_id = channel_details.solo_machine_channel_id.clone();
    let tendermint_channel_id = channel_details.tendermint_channel_id.clone();

    let solo_machine_port_id = channel_details.solo_machine_port_id.clone();
    let tendermint_port_id = channel_details.tendermint_port_id.clone();

    for packet in packets {
        let mut chain_state = context
            .storage()
            .get_chain_state(&chain_state.id)
            .await?
            .ok_or_else(|| anyhow!("chain details for {} not found", chain_state.id))?;

        ensure!(
            tendermint_port_id.to_string() == packet.source_port,
            "invalid source port id"
        );
        ensure!(
            tendermint_channel_id.to_string() == packet.source_channel,
            "invalid source channel id"
        );
        ensure!(
            solo_machine_port_id.to_string() == packet.destination_port,
            "invalid destination port id"
        );
        ensure!(
            solo_machine_channel_id.to_string() == packet.destination_channel,
            "invalid destination channel id"
        );

        ensure!(
            port_id == &solo_machine_port_id,
            "solo machine port id is not the same as given port id"
        );

        let msg = transaction_builder::msg_acknowledgement(
            context,
            &mut chain_state,
            port_id,
            packet,
            memo.clone(),
            request_id.as_deref(),
        )
        .await?;

        let response = context
            .rpc_client()
            .broadcast_tx(&chain_state.config.rpc_addr, msg)
            .await?;

        context.storage().update_chain_state(&chain_state).await?;

        ensure_response_success(&response)?;
    }

    Ok(())
}
