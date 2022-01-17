use anyhow::{anyhow, Result};
use primitive_types::U256;

use crate::{
    event::{Event, EventHandler},
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::Storage,
    tendermint::{JsonRpcClient, TendermintClient},
    transaction_builder,
    types::{
        ics::core::ics24_host::identifier::{ChainId, Identifier},
        operation::OperationType,
    },
};

use super::common::{ensure_response_success, extract_attribute};

/// Mints tokens on given chain
pub async fn mint_tokens<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    amount: U256,
    denom: Identifier,
    receiver: Option<String>,
    memo: String,
) -> Result<String>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: JsonRpcClient,
{
    let address = context.signer().to_account_address(&chain_id).await?;
    let receiver = receiver.unwrap_or_else(|| address.clone());

    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let msg = transaction_builder::msg_token_send(
        context,
        &mut chain_state,
        amount,
        &denom,
        receiver.clone(),
        memo,
        request_id.as_deref(),
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    let transaction_hash = ensure_response_success(&response)?;

    context.storage().update_chain_state(&chain_state).await?;

    let success: bool = extract_attribute(
        &response.deliver_tx.events,
        "fungible_token_packet",
        "success",
    )?
    .parse()?;

    if success {
        context
            .storage()
            .add_operation(
                request_id.as_deref(),
                &chain_state.id,
                &address,
                &denom,
                &amount,
                OperationType::Mint,
                &transaction_hash,
            )
            .await?;

        context
            .handle_event(Event::TokensMinted {
                chain_id,
                request_id,
                to_address: receiver,
                amount,
                denom,
                transaction_hash: transaction_hash.clone(),
            })
            .await?;

        Ok(transaction_hash)
    } else {
        let error = extract_attribute(
            &response.deliver_tx.events,
            "write_acknowledgement",
            "packet_ack",
        )?;

        Err(anyhow!(
            "Failed to mint tokens on IBC enabled chain: {}",
            error
        ))
    }
}