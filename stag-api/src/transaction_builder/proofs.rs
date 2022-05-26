use anyhow::{anyhow, Result};
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::{
    ClientStateData, ConnectionStateData, ConsensusStateData, DataType, HeaderData, SignBytes,
};
use prost_types::Any;

use crate::{
    signer::Signer,
    stag::StagContext,
    storage::{Storage, Transaction},
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::{
                identifier::{ClientId, ConnectionId},
                path::{ClientStatePath, ConnectionPath, ConsensusStatePath},
            },
        },
        proto_util::{proto_encode, AnyConvert},
    },
};

use super::{
    common::to_u64_timestamp,
    signing::{sign, timestamped_sign},
};

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
