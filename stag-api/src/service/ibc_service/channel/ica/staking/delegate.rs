use anyhow::{anyhow, bail, Result};
use primitive_types::U256;

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{
        ensure_response_success, extract_attribute, get_packet_acknowledgement,
    },
    signer::Signer,
    stag::{StagContext, WithTransaction},
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
        operation::OperationType,
    },
};

/// Delegates some tokens from ICA account on host chain to given validator address
pub async fn delegate<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    validator_address: String,
    amount: U256,
    denom: Identifier,
    memo: String,
) -> Result<String>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let transaction_context = context.with_transaction().await?;

    let mut chain_state = transaction_context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let solo_machine_port_id = PortId::ica_controller();

    let msg = transaction_builder::ica::staking::msg_delegate(
        context,
        &mut chain_state,
        &solo_machine_port_id,
        validator_address.clone(),
        amount,
        &denom,
        memo.clone(),
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

    if get_packet_acknowledgement(&response.deliver_tx.events).is_err() {
        let error = extract_attribute(&response.deliver_tx.events, "ics27_packet", "error")?;
        bail!("Failed to execute ICA transaction: {}", error);
    }

    context
        .storage()
        .add_operation(
            request_id.as_deref(),
            &chain_state.id,
            &solo_machine_port_id,
            &OperationType::IcaDelegate {
                validator_address: validator_address.clone(),
                denom: denom.clone(),
                amount,
            },
            &transaction_hash,
        )
        .await?;

    context
        .handle_event(Event::TokensDelegatedFromIca {
            chain_id,
            request_id,
            validator_address,
            amount,
            denom,
            transaction_hash: transaction_hash.clone(),
        })
        .await?;

    Ok(transaction_hash)
}
