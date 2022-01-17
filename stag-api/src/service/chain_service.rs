use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use tendermint::node::Id as NodeId;

use crate::{
    event::{Event, EventHandler},
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    types::{
        chain_state::{ChainConfig, ChainKey, ChainState},
        ics::core::ics24_host::identifier::{ChainId, Identifier},
        operation::Operation,
    },
};

/// Add details of an IBC enabled chain
pub async fn add_chain<C>(context: &C, config: &ChainConfig) -> Result<ChainId>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
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

    context
        .storage()
        .add_chain_state(chain_id.clone(), node_id, config.clone())
        .await?;
    context
        .storage()
        .add_chain_key(&chain_id, &public_key)
        .await?;

    context
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
pub async fn get_ibc_denom<C>(context: &C, chain_id: &ChainId, denom: &Identifier) -> Result<String>
where
    C: StagContext,
    C::Storage: Storage,
{
    let chain = get_chain(context, chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details not found when computing ibc denom"))?;
    chain.get_ibc_denom(denom)
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

/// Fetches the balance of a token on solo machine client on given chain
pub async fn get_balance<C>(context: &C, chain_id: &ChainId, denom: &Identifier) -> Result<Decimal>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
{
    let chain_state = get_chain(context, chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details not found when computing balance"))?;
    chain_state.get_balance(context.signer(), denom).await
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
