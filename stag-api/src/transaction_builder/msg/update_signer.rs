#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::Header as SoloMachineHeader;
use anyhow::{anyhow, bail, Result};
#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::Header as SoloMachineHeader;
use cosmos_sdk_proto::{cosmos::tx::v1beta1::TxRaw, ibc::core::client::v1::MsgUpdateClient};

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    transaction_builder::{common::to_u64_timestamp, proofs::get_header_proof, tx::build},
    types::{chain_state::ChainState, proto_util::AnyConvert, public_key::PublicKey},
};

/// Creates a message for updating solo machine client on IBC enabled chain
pub async fn msg_update_solo_machine_client<C>(
    context: &C,
    chain_state: &mut ChainState,
    new_public_key: &PublicKey,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    if chain_state.connection_details.is_none() {
        bail!(
            "connection details not found for chain with id {}",
            chain_state.id
        );
    }

    #[cfg(not(feature = "solo-machine-v3"))]
    let sequence = chain_state.sequence.into();
    let any_public_key = new_public_key.to_any()?;

    let signature = get_header_proof(
        context,
        chain_state,
        any_public_key.clone(),
        chain_state.config.diversifier.clone(),
        request_id,
    )
    .await?;

    let header = SoloMachineHeader {
        #[cfg(not(feature = "solo-machine-v3"))]
        sequence,
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
        signature,
        new_public_key: Some(any_public_key),
        new_diversifier: chain_state.config.diversifier.clone(),
    };

    let any_header = header.to_any()?;

    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details not found for chain with id {}",
            chain_state.id
        )
    })?;

    let message = MsgUpdateClient {
        client_id: connection_details.tendermint_client_id.to_string(),
        header: Some(any_header),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
