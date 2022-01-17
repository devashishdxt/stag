use std::collections::HashMap;

use anyhow::{anyhow, ensure, Context, Result};
use cosmos_sdk_proto::ibc::core::{channel::v1::Packet, client::v1::Height};
use tendermint::abci::{
    tag::{Key, Tag},
    Event as AbciEvent,
};
use tendermint_rpc::endpoint::broadcast::tx_commit::Response as TxCommitResponse;

use crate::{
    signer::Signer,
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    transaction_builder,
    types::{chain_state::ChainState, ics::core::ics02_client::height::IHeight},
};

pub fn extract_attribute(events: &[AbciEvent], event_type: &str, key: &str) -> Result<String> {
    let mut attribute = None;

    for event in events {
        if event.type_str == event_type {
            attribute = Some(get_attribute(&event.attributes, key)?);
        }
    }

    attribute.ok_or_else(|| {
        anyhow!(
            "{}:{} not found in tendermint response events: {:?}",
            event_type,
            key,
            events
        )
    })
}

fn get_attribute(tags: &[Tag], key: &str) -> Result<String> {
    let key: Key = key
        .parse()
        .map_err(|e| anyhow!("unable to parse attribute key `{}`: {}", key, e))?;

    for tag in tags {
        if tag.key == key {
            return Ok(tag.value.to_string());
        }
    }

    Err(anyhow!("{} not found in tags: {:?}", key, tags))
}

pub fn ensure_response_success(response: &TxCommitResponse) -> Result<String> {
    ensure!(
        response.check_tx.code.is_ok(),
        "check_tx response contains error code: {}",
        response.check_tx.log
    );

    ensure!(
        response.deliver_tx.code.is_ok(),
        "deliver_tx response contains error code: {}",
        response.deliver_tx.log
    );

    Ok(response.hash.to_string())
}

pub fn extract_packets(response: &TxCommitResponse) -> Result<Vec<Packet>> {
    let mut packets = vec![];

    for event in response.deliver_tx.events.iter() {
        if event.type_str == "send_packet" {
            let mut attributes = HashMap::new();

            for tag in event.attributes.iter() {
                attributes.insert(tag.key.to_string(), tag.value.to_string());
            }

            let packet = Packet {
                sequence: attributes
                    .remove("packet_sequence")
                    .ok_or_else(|| anyhow!("`packet_sequence` is missing from packet data"))?
                    .parse()
                    .context("invalid `packet_sequence`")?,
                source_port: attributes
                    .remove("packet_src_port")
                    .ok_or_else(|| anyhow!("`packet_src_port` is missing from packet data"))?,
                source_channel: attributes
                    .remove("packet_src_channel")
                    .ok_or_else(|| anyhow!("`packet_src_channel` is missing from packet data"))?,
                destination_port: attributes
                    .remove("packet_dst_port")
                    .ok_or_else(|| anyhow!("`packet_dst_port` is missing from packet data"))?,
                destination_channel: attributes
                    .remove("packet_dst_channel")
                    .ok_or_else(|| anyhow!("`packet_dst_channel` is missing from packet data"))?,
                data: attributes
                    .remove("packet_data")
                    .ok_or_else(|| anyhow!("`packet_data` is missing from packet data"))?
                    .into_bytes(),
                timeout_height: Some(
                    Height::from_str(&attributes.remove("packet_timeout_height").ok_or_else(
                        || anyhow!("`packet_timeout_height` is missing from packet data"),
                    )?)
                    .context("invalid `packet_timeout_height`")?,
                ),
                timeout_timestamp: attributes
                    .remove("packet_timeout_timestamp")
                    .ok_or_else(|| {
                        anyhow!("`packet_timeout_timestamp` is missing from packet data")
                    })?
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
    let connection_details = chain_state.connection_details.clone().ok_or_else(|| {
        anyhow!(
            "connection details for chain with id {} are missing",
            chain_state.id
        )
    })?;
    ensure!(
        connection_details.tendermint_channel_id.is_some(),
        "can't find tendermint channel, channel is already closed"
    );
    ensure!(
        connection_details.solo_machine_channel_id.is_some(),
        "can't find solo machine channel, channel is already closed"
    );
    let solo_machine_channel_id = connection_details.solo_machine_channel_id.as_ref().unwrap();
    let tendermint_channel_id = connection_details.tendermint_channel_id.as_ref().unwrap();

    for packet in packets {
        let mut chain_state = context
            .storage()
            .get_chain_state(&chain_state.id)
            .await?
            .ok_or_else(|| anyhow!("chain details for {} not found", chain_state.id))?;

        ensure!(
            chain_state.config.port_id.to_string() == packet.source_port,
            "invalid source port id"
        );
        ensure!(
            solo_machine_channel_id.to_string() == packet.source_channel,
            "invalid source channel id"
        );
        ensure!(
            chain_state.config.port_id.to_string() == packet.destination_port,
            "invalid destination port id"
        );
        ensure!(
            tendermint_channel_id.to_string() == packet.destination_channel,
            "invalid destination channel id"
        );

        let msg = transaction_builder::msg_token_receive_ack(
            context,
            &mut chain_state,
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
