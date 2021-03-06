use anyhow::{anyhow, bail, Result};

use crate::{
    event::{Event, EventHandler},
    signer::Signer,
    stag::{StagContext, WithTransaction},
    storage::{Storage, Transaction, TransactionProvider},
    tendermint::TendermintClient,
    types::{
        chain_state::ConnectionDetails,
        ics::core::ics24_host::identifier::{ChainId, PortId},
    },
};

use super::{
    channel::{self, ica, transfer},
    client::create_client,
    connection::establish_connection,
};

/// Creates IBC client and connection with an IBC enabled chain
pub async fn connect<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    memo: String,
    force: bool,
) -> Result<()>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let context = context.with_transaction().await?;

    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    if !chain_state.is_connected() || force {
        chain_state.sequence = 1;

        let (solo_machine_client_id, tendermint_client_id) =
            create_client(&context, &chain_state, request_id.as_deref(), memo.clone()).await?;

        let (solo_machine_connection_id, tendermint_connection_id) = establish_connection(
            &context,
            &mut chain_state,
            request_id.as_deref(),
            memo.clone(),
            &solo_machine_client_id,
            &tendermint_client_id,
        )
        .await?;

        let connection_details = ConnectionDetails {
            solo_machine_client_id,
            tendermint_client_id,
            solo_machine_connection_id,
            tendermint_connection_id,
            channels: Default::default(),
        };

        chain_state.connection_details = Some(connection_details);

        context.storage().update_chain_state(&chain_state).await?;

        let (_, transaction, _, event_handler) = context.unwrap();
        transaction.done().await?;

        event_handler
            .handle_event(Event::ConnectionEstablished {
                chain_id,
                connection_details: chain_state.connection_details.as_ref().unwrap().clone(),
            })
            .await
    } else {
        Err(anyhow!("chain {} is already connected", chain_id))
    }
}

/// Creates IBC transfer channel with an IBC enabled chain
pub async fn create_transfer_channel<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    memo: String,
) -> Result<()>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let context = context.with_transaction().await?;

    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let (solo_machine_connection_id, tendermint_connection_id) =
        match chain_state.connection_details {
            Some(ref connection_details) => (
                connection_details.solo_machine_connection_id.clone(),
                connection_details.tendermint_connection_id.clone(),
            ),
            None => bail!("chain {} is not connected", chain_id),
        };

    if !chain_state.is_connected() {
        bail!("chain {} is not connected", chain_id);
    }

    let channel_details = transfer::open_channel(
        &context,
        &mut chain_state,
        request_id.as_deref(),
        memo,
        &solo_machine_connection_id,
        &tendermint_connection_id,
    )
    .await?;

    let connection_details = chain_state.connection_details.as_mut().unwrap();

    connection_details.channels.insert(
        channel_details.solo_machine_port_id.clone(),
        channel_details.clone(),
    );

    context.storage().update_chain_state(&chain_state).await?;

    let (_, transaction, _, event_handler) = context.unwrap();
    transaction.done().await?;

    event_handler
        .handle_event(Event::ChannelCreated {
            chain_id,
            channel_details,
        })
        .await
}

/// Creates ICA (Interchain Accounts) channel with an IBC enabled chain
pub async fn create_ica_channel<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    memo: String,
) -> Result<()>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let context = context.with_transaction().await?;

    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let (solo_machine_connection_id, tendermint_connection_id) =
        match chain_state.connection_details {
            Some(ref connection_details) => (
                connection_details.solo_machine_connection_id.clone(),
                connection_details.tendermint_connection_id.clone(),
            ),
            None => bail!("chain {} is not connected", chain_id),
        };

    if !chain_state.is_connected() {
        bail!("chain {} is not connected", chain_id);
    }

    let channel_details = ica::open_channel(
        &context,
        &mut chain_state,
        request_id.as_deref(),
        memo,
        &solo_machine_connection_id,
        &tendermint_connection_id,
    )
    .await?;

    let connection_details = chain_state.connection_details.as_mut().unwrap();

    connection_details.channels.insert(
        channel_details.solo_machine_port_id.clone(),
        channel_details.clone(),
    );

    context.storage().update_chain_state(&chain_state).await?;

    let (_, transaction, _, event_handler) = context.unwrap();
    transaction.done().await?;

    event_handler
        .handle_event(Event::ChannelCreated {
            chain_id,
            channel_details,
        })
        .await
}

pub async fn close_channel<C>(
    context: &C,
    chain_id: ChainId,
    port_id: &PortId,
    request_id: Option<String>,
    memo: String,
) -> Result<()>
where
    C: StagContext + WithTransaction,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let context = context.with_transaction().await?;

    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    if !chain_state.is_connected() {
        bail!("chain {} is not connected", chain_id);
    }

    if !chain_state.has_channel(port_id) {
        bail!("channel with port id {} is not created", port_id);
    }

    channel::close_channel(
        &context,
        &mut chain_state,
        port_id,
        memo,
        request_id.as_deref(),
    )
    .await?;

    let connection_details = chain_state.connection_details.as_mut().unwrap();
    let channel_details = connection_details.channels.remove(port_id).unwrap();

    context.storage().update_chain_state(&chain_state).await?;

    let (_, transaction, _, event_handler) = context.unwrap();
    transaction.done().await?;

    event_handler
        .handle_event(Event::ChannelClosed {
            chain_id,
            channel_details,
        })
        .await
}
