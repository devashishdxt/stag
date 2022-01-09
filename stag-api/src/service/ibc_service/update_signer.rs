use anyhow::{anyhow, Result};

use crate::{
    event::{Event, EventHandler},
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::TendermintClient,
    transaction_builder,
    types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKey},
};

use super::common::ensure_response_success;

pub async fn update_signer<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    new_public_key: PublicKey,
    memo: String,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let transaction = context.storage().transaction(&[
        "get_chain_state",
        "update_chain_state",
        "add_chain_key",
    ])?;

    let mut chain_state = transaction
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    transaction
        .add_chain_key(&chain_id, &new_public_key.to_string())
        .await?;

    let msg = transaction_builder::msg_update_solo_machine_client(
        context,
        &mut chain_state,
        Some(&new_public_key),
        memo,
        request_id.as_deref(),
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    transaction.update_chain_state(&chain_state).await?;

    transaction.done().await?;

    context
        .handle_event(Event::SignerUpdated {
            chain_id,
            old_public_key: context.signer().get_public_key(&chain_state.id)?,
            new_public_key,
        })
        .await
}
