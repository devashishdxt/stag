use anyhow::{anyhow, Result};
use primitive_types::U256;

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::{
        common::ensure_response_success,
        packet::{extract_packets, process_packets},
    },
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
        operation::OperationType,
    },
};

/// Burns tokens on given chain
pub async fn burn_tokens<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    amount: U256,
    denom: Identifier,
    memo: String,
) -> Result<String>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let address = context.signer().to_account_address(&chain_id).await?;
    let chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let msg = transaction_builder::transfer::msg_burn(
        context,
        &chain_state,
        amount,
        &denom,
        address.clone(),
        memo.clone(),
        request_id.as_deref(),
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    let transaction_hash = ensure_response_success(&response)?;

    let port_id = PortId::transfer();

    context
        .storage()
        .add_operation(
            request_id.as_deref(),
            &chain_state.id,
            &port_id,
            &OperationType::Burn {
                from: address.clone(),
                denom: denom.clone(),
                amount,
            },
            &transaction_hash,
        )
        .await?;

    context
        .handle_event(Event::TokensBurnt {
            chain_id,
            request_id: request_id.clone(),
            from_address: address,
            amount,
            denom,
            transaction_hash: transaction_hash.clone(),
        })
        .await?;

    if let Err(e) = process_packets(
        context,
        &chain_state,
        &port_id,
        extract_packets(&response)?,
        memo,
        request_id,
    )
    .await
    {
        // Create a warning instead of returning an error because IBC transfer is successful even if processing of
        // packets (i.e., sending acks) fails
        context
            .handle_event(Event::Warning {
                message: format!("Failed to process packets: {}", e),
            })
            .await?;
    }

    Ok(transaction_hash)
}
