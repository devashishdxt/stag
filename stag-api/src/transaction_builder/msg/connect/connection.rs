use anyhow::{anyhow, Result};
use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::core::{
        client::v1::Height,
        commitment::v1::MerklePrefix,
        connection::v1::{
            Counterparty as ConnectionCounterparty, MsgConnectionOpenAck, MsgConnectionOpenInit,
            Version as ConnectionVersion,
        },
    },
};

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::{Storage, Transaction},
    transaction_builder::{
        proofs::{get_client_proof, get_connection_proof, get_consensus_proof},
        tx::build,
    },
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::identifier::{ClientId, ConnectionId},
        },
        proto_util::AnyConvert,
    },
};

/// Creates a message for opening a connection on IBC enabled chain
pub async fn msg_connection_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    client_id: &ClientId,
    counterparty_client_id: &ClientId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let message = MsgConnectionOpenInit {
        client_id: client_id.to_string(),
        counterparty: Some(ConnectionCounterparty {
            client_id: counterparty_client_id.to_string(),
            connection_id: "".to_string(),
            prefix: Some(MerklePrefix {
                key_prefix: "ibc".as_bytes().to_vec(),
            }),
        }),
        version: Some(ConnectionVersion {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        }),
        delay_period: 0,
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

#[allow(clippy::too_many_arguments)]
/// Creates a message for acknowledginging a connection open on IBC enabled chain
pub async fn msg_connection_open_ack<C>(
    context: &C,
    chain_state: &mut ChainState,
    connection_id: &ConnectionId,
    counterparty_client_id: &ClientId,
    counterparty_connection_id: &ConnectionId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
{
    let tendermint_client_state = context
        .storage()
        .get_tendermint_client_state(counterparty_client_id)
        .await?
        .ok_or_else(|| anyhow!("client for client id {} not found", counterparty_client_id))?;

    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_try =
        get_connection_proof(context, chain_state, counterparty_connection_id, request_id).await?;

    let proof_client =
        get_client_proof(context, chain_state, counterparty_client_id, request_id).await?;

    let proof_consensus =
        get_consensus_proof(context, chain_state, counterparty_client_id, request_id).await?;

    let message = MsgConnectionOpenAck {
        connection_id: connection_id.to_string(),
        counterparty_connection_id: counterparty_connection_id.to_string(),
        version: Some(ConnectionVersion {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        }),
        client_state: Some(tendermint_client_state.to_any()?),
        proof_height: Some(proof_height),
        proof_try,
        proof_client,
        proof_consensus,
        consensus_height: tendermint_client_state.latest_height,
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
