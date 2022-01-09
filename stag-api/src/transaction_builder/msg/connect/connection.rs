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
    storage::Transaction,
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

pub async fn msg_connection_open_init<C>(
    context: &C,
    chain_state: &ChainState,
    solo_machine_client_id: &ClientId,
    tendermint_client_id: &ClientId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let message = MsgConnectionOpenInit {
        client_id: solo_machine_client_id.to_string(),
        counterparty: Some(ConnectionCounterparty {
            client_id: tendermint_client_id.to_string(),
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
        signer: context.signer().to_account_address(&chain_state.id)?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

#[allow(clippy::too_many_arguments)]
pub async fn msg_connection_open_ack<C, T>(
    context: &C,
    transaction: &T,
    chain_state: &mut ChainState,
    solo_machine_connection_id: &ConnectionId,
    tendermint_client_id: &ClientId,
    tendermint_connection_id: &ConnectionId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    T: Transaction,
{
    let tendermint_client_state = transaction
        .get_tendermint_client_state(tendermint_client_id)
        .await?
        .ok_or_else(|| anyhow!("client for client id {} not found", tendermint_client_id))?;

    let proof_height = Height::new(0, chain_state.sequence.into());

    let proof_try = get_connection_proof(
        context,
        transaction,
        chain_state,
        tendermint_connection_id,
        request_id,
    )
    .await?;

    chain_state.sequence += 1;

    let proof_client = get_client_proof(
        context,
        transaction,
        chain_state,
        tendermint_client_id,
        request_id,
    )
    .await?;

    chain_state.sequence += 1;

    let proof_consensus = get_consensus_proof(
        context,
        transaction,
        chain_state,
        tendermint_client_id,
        request_id,
    )
    .await?;

    chain_state.sequence += 1;

    let message = MsgConnectionOpenAck {
        connection_id: solo_machine_connection_id.to_string(),
        counterparty_connection_id: tendermint_connection_id.to_string(),
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
        signer: context.signer().to_account_address(&chain_state.id)?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
