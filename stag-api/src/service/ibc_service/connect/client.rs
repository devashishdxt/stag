use anyhow::{anyhow, Result};

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{ensure_response_success, extract_attribute},
    signer::Signer,
    stag::StagContext,
    storage::Storage,
    tendermint::{LightClient, TendermintClient},
    transaction_builder,
    types::{
        chain_state::ChainState,
        ics::core::{ics02_client::client_type::ClientType, ics24_host::identifier::ClientId},
    },
};

pub async fn create_client<C>(
    context: &C,
    chain_state: &ChainState,
    request_id: Option<&str>,
    memo: String,
) -> Result<(ClientId, ClientId)>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let solo_machine_client_id =
        create_solo_machine_client(context, chain_state, request_id, memo).await?;

    context
        .handle_event(Event::CreatedSoloMachineClient {
            client_id: solo_machine_client_id.clone(),
        })
        .await?;

    let tendermint_client_id = create_tendermint_client(context, chain_state).await?;

    context
        .handle_event(Event::CreatedTendermintClient {
            client_id: tendermint_client_id.clone(),
        })
        .await?;

    Ok((solo_machine_client_id, tendermint_client_id))
}

async fn create_solo_machine_client<C>(
    context: &C,
    chain_state: &ChainState,
    request_id: Option<&str>,
    memo: String,
) -> Result<ClientId>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let msg =
        transaction_builder::msg_create_solo_machine_client(context, chain_state, memo, request_id)
            .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    extract_attribute(&response.deliver_tx.events, "create_client", "client_id")?.parse()
}

async fn create_tendermint_client<C>(context: &C, chain_state: &ChainState) -> Result<ClientId>
where
    C: StagContext,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let light_client = LightClient::new(
        chain_state.config.rpc_addr.clone(),
        context.rpc_client(),
        chain_state,
    )
    .await?;

    let (client_state, consensus_state) =
        transaction_builder::msg_create_tendermint_client(chain_state, &light_client).await?;

    let client_id = ClientId::generate(ClientType::Tendermint);
    let latest_height = client_state
        .latest_height
        .as_ref()
        .ok_or_else(|| anyhow!("latest height cannot be absent in client state"))?;

    context
        .storage()
        .add_tendermint_client_state(&client_id, &client_state)
        .await?;
    context
        .storage()
        .add_tendermint_consensus_state(&client_id, latest_height, &consensus_state)
        .await?;

    Ok(client_id)
}
