use anyhow::{anyhow, ensure, Result};
use cosmos_sdk_proto::ibc::core::channel::v1::Packet;
use prost_types::Any;

use crate::{
    signer::Signer,
    stag::StagContext,
    storage::{Storage, Transaction},
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics04_channel::packet::IPacket,
            ics24_host::{
                identifier::{ChannelId, ClientId, ConnectionId},
                path::{
                    ChannelPath, ClientStatePath, ConnectionPath, ConsensusStatePath,
                    PacketAcknowledgementPath, PacketCommitmentPath,
                },
            },
        },
        proto::ibc::lightclients::solomachine::v2::{
            ChannelStateData, ClientStateData, ConnectionStateData, ConsensusStateData, DataType,
            HeaderData, PacketAcknowledgementData, PacketCommitmentData, SignBytes,
        },
        proto_util::{proto_encode, AnyConvert},
    },
};

use super::{
    common::to_u64_timestamp,
    signing::{sign, timestamped_sign},
};

pub async fn get_packet_acknowledgement_proof<C>(
    context: &C,
    chain_state: &ChainState,
    acknowledgement: Vec<u8>,
    packet_sequence: u64,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details for chain with id {} not found",
            chain_state.id
        )
    })?;
    ensure!(
        connection_details.tendermint_channel_id.is_some(),
        "can't find tendermint channel, channel is already closed"
    );
    let mut acknowledgement_path = PacketAcknowledgementPath::new(
        &chain_state.config.port_id,
        connection_details.tendermint_channel_id.as_ref().unwrap(),
        packet_sequence,
    );
    acknowledgement_path.apply_prefix(&"ibc".parse().unwrap());

    let acknowledgement_data = PacketAcknowledgementData {
        path: acknowledgement_path.into_bytes(),
        acknowledgement,
    };

    let acknowledgement_data_bytes = proto_encode(&acknowledgement_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::PacketAcknowledgement.into(),
        data: acknowledgement_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_packet_commitment_proof<C>(
    context: &C,
    chain_state: &ChainState,
    packet: &Packet,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let commitment_bytes = packet.commitment_bytes()?;

    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details for chain with id {} not found",
            chain_state.id
        )
    })?;
    ensure!(
        connection_details.tendermint_channel_id.is_some(),
        "can't find tendermint channel id, channel is already closed"
    );
    let mut commitment_path = PacketCommitmentPath::new(
        &chain_state.config.port_id,
        connection_details.tendermint_channel_id.as_ref().unwrap(),
        chain_state.packet_sequence.into(),
    );
    commitment_path.apply_prefix(&"ibc".parse().unwrap());

    let packet_commitment_data = PacketCommitmentData {
        path: commitment_path.into_bytes(),
        commitment: commitment_bytes,
    };

    let packet_commitment_data_bytes = proto_encode(&packet_commitment_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::PacketCommitment.into(),
        data: packet_commitment_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_channel_proof<C>(
    context: &C,
    chain_state: &ChainState,
    channel_id: &ChannelId,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let channel = context
        .storage()
        .get_channel(&chain_state.config.port_id, channel_id)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "channel with port id {} and channel id {} not found",
                chain_state.config.port_id,
                channel_id
            )
        })?;

    let mut channel_path = ChannelPath::new(&chain_state.config.port_id, channel_id);
    channel_path.apply_prefix(&"ibc".parse().unwrap());

    let channel_state_data = ChannelStateData {
        path: channel_path.into_bytes(),
        channel: Some(channel),
    };

    let channel_state_data_bytes = proto_encode(&channel_state_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::ChannelState.into(),
        data: channel_state_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_connection_proof<C>(
    context: &C,
    chain_state: &ChainState,
    connection_id: &ConnectionId,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let connection = context
        .storage()
        .get_connection(connection_id)
        .await?
        .ok_or_else(|| anyhow!("connection with id {} not found", connection_id))?;

    let mut connection_path = ConnectionPath::new(connection_id);
    connection_path.apply_prefix(&"ibc".parse().unwrap());

    let connection_state_data = ConnectionStateData {
        path: connection_path.into_bytes(),
        connection: Some(connection),
    };

    let connection_state_data_bytes = proto_encode(&connection_state_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::ConnectionState.into(),
        data: connection_state_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_client_proof<C>(
    context: &C,
    chain_state: &ChainState,
    client_id: &ClientId,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let client_state = context
        .storage()
        .get_tendermint_client_state(client_id)
        .await?
        .ok_or_else(|| anyhow!("client with id {} not found", client_id))?
        .to_any()?;

    let mut client_state_path = ClientStatePath::new(client_id);
    client_state_path.apply_prefix(&"ibc".parse().unwrap());

    let client_state_data = ClientStateData {
        path: client_state_path.into_bytes(),
        client_state: Some(client_state),
    };

    let client_state_data_bytes = proto_encode(&client_state_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::ClientState.into(),
        data: client_state_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_consensus_proof<C>(
    context: &C,
    chain_state: &ChainState,
    client_id: &ClientId,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
{
    let client_state = context
        .storage()
        .get_tendermint_client_state(client_id)
        .await?
        .ok_or_else(|| anyhow!("client with id {} not found", client_id))?;

    let height = client_state
        .latest_height
        .ok_or_else(|| anyhow!("client state does not contain latest height"))?;

    let consensus_state = context
        .storage()
        .get_tendermint_consensus_state(client_id, &height)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "consensus state with id {} and height {} not found",
                client_id,
                height.to_string(),
            )
        })?
        .to_any()?;

    let mut consensus_state_path = ConsensusStatePath::new(client_id, &height);
    consensus_state_path.apply_prefix(&"ibc".parse().unwrap());

    let consensus_state_data = ConsensusStateData {
        path: consensus_state_path.into_bytes(),
        consensus_state: Some(consensus_state),
    };

    let consensus_state_data_bytes = proto_encode(&consensus_state_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::ConsensusState.into(),
        data: consensus_state_data_bytes,
    };

    timestamped_sign(context, chain_state, sign_bytes, request_id).await
}

pub async fn get_header_proof<C>(
    context: &C,
    chain_state: &ChainState,
    new_public_key: Any,
    new_diversifier: String,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let header_data = HeaderData {
        new_pub_key: Some(new_public_key),
        new_diversifier,
    };

    let header_data_bytes = proto_encode(&header_data)?;

    let sign_bytes = SignBytes {
        sequence: chain_state.sequence.into(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        diversifier: chain_state.config.diversifier.to_owned(),
        data_type: DataType::Header.into(),
        data: header_data_bytes,
    };

    sign(context, request_id, &chain_state.id, sign_bytes).await
}
