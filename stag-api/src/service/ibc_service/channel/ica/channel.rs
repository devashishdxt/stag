#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
use anyhow::Context;
use anyhow::{anyhow, Result};
use cosmos_sdk_proto::ibc::core::channel::v1::{
    query_client::QueryClient as ChannelQueryClient, Channel, Counterparty as ChannelCounterparty,
    Order as ChannelOrder, QueryChannelRequest, State as ChannelState,
};
use serde_json::json;
#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
use tonic::transport::Channel;
#[cfg(feature = "wasm")]
use tonic_web_wasm_client::Client;
use url::Url;

use crate::{
    event::{Event, EventHandler},
    service::ibc_service::common::{ensure_response_success, extract_attribute},
    signer::Signer,
    stag::StagContext,
    storage::{Storage, Transaction},
    tendermint::TendermintClient,
    transaction_builder,
    types::{
        chain_state::{ChainState, ChannelDetails},
        ics::core::ics24_host::identifier::{ChannelId, ConnectionId, PortId},
    },
};

pub async fn open_channel<C>(
    context: &C,
    chain_state: &mut ChainState,
    request_id: Option<&str>,
    memo: String,
    solo_machine_connection_id: &ConnectionId,
    tendermint_connection_id: &ConnectionId,
) -> Result<ChannelDetails>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Transaction,
    C::RpcClient: TendermintClient,
{
    let solo_machine_port_id = PortId::ica_controller(context.signer(), &chain_state.id).await?;
    let tendermint_port_id = PortId::ica_host();

    let solo_machine_version = serde_json::to_string(&json!({
       "version": "ics27-1",
       "controller_connection_id": solo_machine_connection_id,
       "host_connection_id": tendermint_connection_id,
       "address": "",
       "encoding": "proto3",
       "tx_type": "sdk_multi_msg",
    }))?;

    let solo_machine_channel_id = channel_open_init(
        context,
        &solo_machine_port_id,
        tendermint_connection_id,
        &tendermint_port_id,
        solo_machine_version.clone(),
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnSoloMachine {
            channel_id: solo_machine_channel_id.clone(),
            port_id: solo_machine_port_id.clone(),
        })
        .await?;

    let tendermint_channel_id = channel_open_try(
        context,
        chain_state,
        &solo_machine_channel_id,
        &solo_machine_port_id,
        &tendermint_port_id,
        solo_machine_version,
        memo.clone(),
        request_id,
    )
    .await?;

    context
        .handle_event(Event::InitializedChannelOnTendermint {
            channel_id: tendermint_channel_id.clone(),
            port_id: tendermint_port_id.clone(),
        })
        .await?;

    channel_open_ack(
        context,
        chain_state,
        solo_machine_connection_id,
        &solo_machine_channel_id,
        &solo_machine_port_id,
        &tendermint_channel_id,
        &tendermint_port_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedChannelOnSoloMachine {
            channel_id: solo_machine_channel_id.clone(),
            port_id: solo_machine_port_id.clone(),
        })
        .await?;

    channel_open_confirm(
        context,
        chain_state,
        &solo_machine_port_id,
        &solo_machine_channel_id,
        &tendermint_port_id,
        &tendermint_channel_id,
        memo,
        request_id,
    )
    .await?;

    context
        .handle_event(Event::ConfirmedChannelOnTendermint {
            channel_id: tendermint_channel_id.clone(),
            port_id: tendermint_port_id.clone(),
        })
        .await?;

    Ok(ChannelDetails {
        packet_sequence: 1,
        solo_machine_port_id,
        tendermint_port_id,
        solo_machine_channel_id,
        tendermint_channel_id,
    })
}

async fn channel_open_init<C>(
    context: &C,
    solo_machine_port_id: &PortId,
    tendermint_connection_id: &ConnectionId,
    tendermint_port_id: &PortId,
    solo_machine_version: String,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Storage: Storage,
{
    let channel_id = ChannelId::generate();

    let channel = Channel {
        state: ChannelState::Init.into(),
        ordering: ChannelOrder::Ordered.into(),
        counterparty: Some(ChannelCounterparty {
            port_id: tendermint_port_id.to_string(),
            channel_id: "".to_string(),
        }),
        connection_hops: vec![tendermint_connection_id.to_string()],
        version: solo_machine_version,
    };

    context
        .storage()
        .add_channel(solo_machine_port_id, &channel_id, &channel)
        .await?;

    Ok(channel_id)
}

#[allow(clippy::too_many_arguments)]
async fn channel_open_try<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_channel_id: &ChannelId,
    solo_machine_port_id: &PortId,
    tendermint_port_id: &PortId,
    solo_machine_version: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<ChannelId>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_open_try(
        context,
        chain_state,
        tendermint_port_id,
        solo_machine_channel_id,
        solo_machine_port_id,
        solo_machine_version,
        memo,
        request_id,
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    extract_attribute(
        &response.deliver_tx.events,
        "channel_open_try",
        "channel_id",
    )?
    .parse()
}

async fn channel_open_ack<C>(
    context: &C,
    chain_state: &ChainState,
    solo_machine_connection_id: &ConnectionId,
    solo_machine_channel_id: &ChannelId,
    solo_machine_port_id: &PortId,
    tendermint_channel_id: &ChannelId,
    tendermint_port_id: &PortId,
) -> Result<()>
where
    C: StagContext,
    C::Storage: Transaction,
{
    let mut channel = context
        .storage()
        .get_channel(solo_machine_port_id, solo_machine_channel_id)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "channel for channel id ({}) and port id ({}) not found",
                solo_machine_channel_id,
                solo_machine_port_id
            )
        })?;

    channel.set_state(ChannelState::Open);

    let counterparty = channel.counterparty.as_mut().ok_or_else(|| {
        anyhow!(
            "counterparty not set for channel id ({}) and port id ({})",
            solo_machine_channel_id,
            solo_machine_port_id
        )
    })?;

    counterparty.channel_id = tendermint_channel_id.to_string();

    context
        .storage()
        .update_channel(solo_machine_port_id, solo_machine_channel_id, &channel)
        .await?;

    let ica_address =
        get_ica_address(chain_state, tendermint_channel_id, tendermint_port_id).await?;

    context
        .storage()
        .add_ica_address(
            solo_machine_connection_id,
            solo_machine_port_id,
            &ica_address,
        )
        .await?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn channel_open_confirm<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    solo_machine_channel_id: &ChannelId,
    tendermint_port_id: &PortId,
    tendermint_channel_id: &ChannelId,
    memo: String,
    request_id: Option<&str>,
) -> Result<()>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let msg = transaction_builder::msg_channel_open_confirm(
        context,
        chain_state,
        tendermint_port_id,
        tendermint_channel_id,
        solo_machine_port_id,
        solo_machine_channel_id,
        memo,
        request_id,
    )
    .await?;

    let response = context
        .rpc_client()
        .broadcast_tx(&chain_state.config.rpc_addr, msg)
        .await?;

    ensure_response_success(&response)?;

    Ok(())
}

async fn get_ica_address(
    chain_state: &ChainState,
    tendermint_channel_id: &ChannelId,
    tendermint_port_id: &PortId,
) -> Result<String> {
    let mut query_client = get_channel_query_client(chain_state.config.grpc_addr.clone()).await?;

    let channel = query_client
        .channel(QueryChannelRequest {
            channel_id: tendermint_channel_id.to_string(),
            port_id: tendermint_port_id.to_string(),
        })
        .await?
        .into_inner()
        .channel
        .ok_or_else(|| {
            anyhow!(
                "tendermint channel not found with id {} and port {}",
                tendermint_channel_id,
                tendermint_port_id
            )
        })?;

    let version: serde_json::Value = serde_json::from_str(&channel.version)?;
    let ica_address = version
        .get("address")
        .ok_or_else(|| {
            anyhow!(
                "address not found in version of tendermint channel with id {} and port {}",
                tendermint_channel_id,
                tendermint_port_id
            )
        })?
        .as_str()
        .ok_or_else(|| {
            anyhow!(
                "unable to convert ICA address to string for tendermint channel with id {} and port {}",
                tendermint_channel_id,
                tendermint_port_id
            )
        })?
        .to_string();

    Ok(ica_address)
}

#[cfg(feature = "wasm")]
async fn get_channel_query_client(grpc_addr: Url) -> Result<ChannelQueryClient<Client>> {
    let mut url = grpc_addr.to_string();

    if url.ends_with('/') {
        url.pop();
    }

    let grpc_client = Client::new(url);
    Ok(ChannelQueryClient::new(grpc_client))
}

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
async fn get_channel_query_client(grpc_addr: Url) -> Result<ChannelQueryClient<Channel>> {
    ChannelQueryClient::connect(grpc_addr.to_string())
        .await
        .context("error when initializing grpc client")
}
