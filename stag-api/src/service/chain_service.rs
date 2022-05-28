use anyhow::{bail, Context, Result};
use rust_decimal::Decimal;
use tendermint::node::Id as NodeId;

use crate::{
    event::{Event, EventHandler},
    signer::{GetPublicKey, Signer},
    stag::{StagContext, WithTransaction},
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::TendermintClient,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
        operation::Operation,
    },
};

/// Add details of an IBC enabled chain
pub async fn add_chain<C>(context: &C, config: &ChainConfig) -> Result<ChainId>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let status = context.rpc_client().status(&config.rpc_addr).await?;

    let chain_id: ChainId = status.node_info.network.to_string().parse()?;
    let node_id: NodeId = status.node_info.id;
    let public_key = context
        .signer()
        .get_public_key(&chain_id)
        .await?
        .to_string();

    let context = context.with_transaction().await?;

    context
        .storage()
        .add_chain_state(chain_id.clone(), node_id, config.clone())
        .await?;
    context
        .storage()
        .add_chain_key(&chain_id, &public_key)
        .await?;

    let (_, transaction, _, event_handler) = context.unwrap();
    transaction.done().await?;

    event_handler
        .handle_event(Event::ChainAdded {
            chain_id: chain_id.clone(),
        })
        .await?;

    Ok(chain_id)
}

/// Fetches details of a chain
pub async fn get_chain<C>(context: &C, chain_id: &ChainId) -> Result<Option<ChainState>>
where
    C: StagContext,
    C::Storage: Storage,
{
    context.storage().get_chain_state(chain_id).await
}

/// Fetches details of all chains
pub async fn get_all_chains<C>(
    context: &C,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ChainState>>
where
    C: StagContext,
    C::Storage: Storage,
{
    context.storage().get_all_chain_states(limit, offset).await
}

/// Returns the final denom of a token on solo machine after sending it on given chain
pub async fn get_ibc_denom<C>(
    context: &C,
    chain_id: &ChainId,
    port_id: &PortId,
    denom: &Identifier,
) -> Result<String>
where
    C: StagContext,
    C::Storage: Storage,
{
    let chain = get_chain(context, chain_id)
        .await?
        .context("chain details not found when computing ibc denom")?;
    chain.get_ibc_denom(port_id, denom)
}

/// Fetches all the public keys associated with solo machine client on given chain
pub async fn get_public_keys<C>(
    context: &C,
    chain_id: &ChainId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ChainKey>>
where
    C: StagContext,
    C::Storage: Storage,
{
    context
        .storage()
        .get_chain_keys(chain_id, limit, offset)
        .await
}

/// Fetches the balance of an IBC  token on solo machine client on given chain
pub async fn get_ibc_balance<C>(
    context: &C,
    chain_id: &ChainId,
    port_id: &PortId,
    denom: &Identifier,
) -> Result<Decimal>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let chain_state = get_chain(context, chain_id)
        .await?
        .context("chain details not found when computing balance")?;
    chain_state
        .get_ibc_balance(context.signer(), port_id, denom)
        .await
}

/// Fetches ICA (Interchain Account) address (on host chain) for given chain
pub async fn get_ica_address<C>(context: &C, chain_id: &ChainId) -> Result<String>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let chain_state = get_chain(context, chain_id)
        .await?
        .context("chain details not found when getting ica address")?;

    let solo_machine_connection_id = match chain_state.connection_details {
        Some(ref details) => &details.solo_machine_connection_id,
        None => bail!("chain is not connected"),
    };

    let solo_machine_port_id = PortId::ica_controller(context.signer(), chain_id).await?;

    context
        .storage()
        .get_ica_address(solo_machine_connection_id, &solo_machine_port_id)
        .await?
        .context("ica channel is not created with chain")
}

/// Fetches transaction history of given chain
pub async fn get_history<C>(
    context: &C,
    chain_id: &ChainId,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<Operation>>
where
    C: StagContext,
    C::Storage: Storage,
{
    context
        .storage()
        .get_operations(chain_id, limit, offset)
        .await
}
