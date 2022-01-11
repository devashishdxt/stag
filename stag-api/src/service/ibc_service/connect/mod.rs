mod channel;
mod client;
mod connection;

use anyhow::{anyhow, bail, Result};

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::connect::connection::establish_connection,
    signer::Signer,
    stag::StagContext,
    storage::{Storage, TransactionProvider},
    tendermint::TendermintClient,
    types::{
        chain_state::{ChainState, ConnectionDetails},
        ics::core::ics24_host::identifier::{ChainId, ConnectionId},
    },
};

use self::{channel::open_channel, client::create_client};

enum ConnectionOpenType {
    /// Open full connection (includes creating client, connection and channel)
    Full,
    /// Only open a new channel
    OnlyChannel {
        /// Connection ID of solo machine client on IBC enabled chain
        solo_machine_connection_id: ConnectionId,
        /// Connection ID of IBC enabled chain on solo machine
        tendermint_connection_id: ConnectionId,
    },
    /// Connection is already open
    AlreadyOpen,
}

pub async fn connect<C>(
    context: &C,
    chain_id: ChainId,
    request_id: Option<String>,
    memo: String,
    force: bool,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: TransactionProvider,
    C::RpcClient: TendermintClient,
{
    let mut chain_state = context
        .storage()
        .get_chain_state(&chain_id)
        .await?
        .ok_or_else(|| anyhow!("chain details for {} not found", chain_id))?;

    let connection_open_type = get_connection_open_type(&chain_state, force);

    match connection_open_type {
        ConnectionOpenType::AlreadyOpen => {
            bail!("connection is already established with given chain")
        }
        ConnectionOpenType::Full => {
            let (solo_machine_client_id, tendermint_client_id) =
                create_client(context, &chain_state, request_id.as_deref(), memo.clone()).await?;

            let (solo_machine_connection_id, tendermint_connection_id) = establish_connection(
                context,
                &mut chain_state,
                request_id.as_deref(),
                memo.clone(),
                &solo_machine_client_id,
                &tendermint_client_id,
            )
            .await?;

            let (solo_machine_channel_id, tendermint_channel_id) = open_channel(
                context,
                &mut chain_state,
                request_id.as_deref(),
                memo,
                &solo_machine_connection_id,
                &tendermint_connection_id,
            )
            .await?;

            let connection_details = ConnectionDetails {
                solo_machine_client_id,
                solo_machine_connection_id,
                solo_machine_channel_id: Some(solo_machine_channel_id),
                tendermint_client_id,
                tendermint_connection_id,
                tendermint_channel_id: Some(tendermint_channel_id),
            };

            chain_state.connection_details = Some(connection_details);

            context.storage().update_chain_state(&chain_state).await?;

            context
                .handle_event(Event::ConnectionEstablished {
                    chain_id,
                    connection_details: chain_state.connection_details.as_ref().unwrap().clone(),
                })
                .await
        }
        ConnectionOpenType::OnlyChannel {
            ref solo_machine_connection_id,
            ref tendermint_connection_id,
        } => {
            let (solo_machine_channel_id, tendermint_channel_id) = open_channel(
                context,
                &mut chain_state,
                request_id.as_deref(),
                memo,
                solo_machine_connection_id,
                tendermint_connection_id,
            )
            .await?;

            {
                let connection_details = chain_state.connection_details.as_mut().unwrap();
                connection_details.solo_machine_channel_id = Some(solo_machine_channel_id);
                connection_details.tendermint_channel_id = Some(tendermint_channel_id);
            }

            context.storage().update_chain_state(&chain_state).await?;

            context
                .handle_event(Event::ConnectionEstablished {
                    chain_id,
                    connection_details: chain_state.connection_details.as_ref().unwrap().clone(),
                })
                .await
        }
    }
}

fn get_connection_open_type(chain_state: &ChainState, force: bool) -> ConnectionOpenType {
    match chain_state.connection_details {
        None => ConnectionOpenType::Full,
        Some(ref detail) => {
            if force {
                ConnectionOpenType::Full
            } else if detail.tendermint_channel_id.is_some() {
                ConnectionOpenType::AlreadyOpen
            } else {
                ConnectionOpenType::OnlyChannel {
                    solo_machine_connection_id: detail.solo_machine_connection_id.clone(),
                    tendermint_connection_id: detail.tendermint_connection_id.clone(),
                }
            }
        }
    }
}
