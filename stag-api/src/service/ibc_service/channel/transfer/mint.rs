use anyhow::{anyhow, Context, Result};
use primitive_types::U256;

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{ensure_response_success, get_packet_acknowledgement},
    signer::{GetPublicKey, Signer},
    stag::{StagContext, WithTransaction},
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::{JsonRpcClient, TendermintClient},
    transaction_builder,
    types::{
        ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
        operation::OperationType,
    },
};

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
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: JsonRpcClient,
{
    let address = context.signer().to_account_address(&chain_id).await?;
    let receiver = receiver.unwrap_or_else(|| address.clone());

    let transaction_context = context.with_transaction().await?;

    let mut chain_state = transaction_context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let msg = transaction_builder::transfer::msg_mint(
        &transaction_context,
        &mut chain_state,
        amount,
        &denom,
        receiver.clone(),
        memo,
        request_id.as_deref(),
    )
    .await?;

    let response = transaction_context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    let transaction_hash = ensure_response_success(&response)?;

    transaction_context
        .storage()
        .update_chain_state(&chain_state)
        .await?;

    let (_, transaction, _, _) = transaction_context.unwrap();
    transaction.done().await?;

    get_packet_acknowledgement(&response.deliver_tx.events)?
        .as_str()
        .context("unable to parse acknowledgement result")?;

    context
        .storage()
        .add_operation(
            request_id.as_deref(),
            &chain_state.id,
            &PortId::transfer(),
            &OperationType::Mint {
                to: address,
                denom: denom.clone(),
                amount,
            },
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
}
