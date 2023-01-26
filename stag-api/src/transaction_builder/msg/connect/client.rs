#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::{
    ClientState as SoloMachineClientState, ConsensusState as SoloMachineConsensusState,
};
use anyhow::{Context, Result};
#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::{
    ClientState as SoloMachineClientState, ConsensusState as SoloMachineConsensusState,
};
use cosmos_sdk_proto::{
    cosmos::{
        staking::v1beta1::{query_client::QueryClient as StakingQueryClient, QueryParamsRequest},
        tx::v1beta1::TxRaw,
    },
    ibc::{
        core::client::v1::{Height, MsgCreateClient},
        lightclients::tendermint::v1::{
            ClientState as TendermintClientState, ConsensusState as TendermintConsensusState,
            Fraction,
        },
    },
};
use prost_types::Duration;
use tendermint::block::Header;
#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
use tonic::transport::Channel;
#[cfg(feature = "wasm")]
use tonic_web_wasm_client::Client;
use url::Url;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    tendermint::{LightClient, TendermintClient},
    transaction_builder::{common::to_u64_timestamp, tx::build},
    types::{
        chain_state::ChainState,
        ics::{
            core::{ics02_client::height::IHeight, ics23_vector_commitments::proof_specs},
            lightclients::tendermint::consensus_state::IConsensusState,
        },
        proto_util::AnyConvert,
    },
};

/// Creates a message for creating a solo machine client on IBC enabled chain
pub async fn msg_create_solo_machine_client<C>(
    context: &C,
    chain_state: &ChainState,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let any_public_key = context
        .signer()
        .get_public_key(&chain_state.id)
        .await?
        .to_any()?;

    let consensus_state = SoloMachineConsensusState {
        public_key: Some(any_public_key),
        diversifier: chain_state.config.diversifier.clone(),
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
    };
    let any_consensus_state = consensus_state.to_any()?;

    let client_state = SoloMachineClientState {
        sequence: chain_state.sequence.into(),
        is_frozen: false,
        consensus_state: Some(consensus_state),
        #[cfg(not(feature = "solo-machine-v3"))]
        allow_update_after_proposal: true,
    };
    let any_client_state = client_state.to_any()?;

    let message = MsgCreateClient {
        client_state: Some(any_client_state),
        consensus_state: Some(any_consensus_state),
        signer: context.signer().to_account_address(&chain_state.id).await?,
    };

    build(context, chain_state, &[message], memo, request_id).await
}

/// Creates tendermint client on solo machine
pub async fn msg_create_tendermint_client<T>(
    chain_state: &ChainState,
    light_client: &LightClient<T>,
) -> Result<(TendermintClientState, TendermintConsensusState)>
where
    T: TendermintClient,
{
    let trust_level = Some(Fraction {
        numerator: *chain_state.config.trust_level.numer(),
        denominator: *chain_state.config.trust_level.denom(),
    });

    let unbonding_period = Some(get_unbonding_period(chain_state).await?);
    let latest_header = get_latest_header(light_client).await?;
    let latest_height = get_block_height(chain_state, &latest_header);

    let client_state = TendermintClientState {
        chain_id: chain_state.id.to_string(),
        trust_level,
        trusting_period: Some(chain_state.config.trusting_period.try_into()?),
        unbonding_period,
        max_clock_drift: Some(chain_state.config.max_clock_drift.try_into()?),
        frozen_height: Some(Height::zero()),
        latest_height: Some(latest_height),
        proof_specs: proof_specs(),
        upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
        allow_update_after_expiry: false,
        allow_update_after_misbehaviour: false,
    };

    let consensus_state = TendermintConsensusState::from_block_header(latest_header);

    Ok((client_state, consensus_state))
}

async fn get_unbonding_period(chain_state: &ChainState) -> Result<Duration> {
    let mut query_client = get_staking_query_client(chain_state.config.grpc_addr.clone()).await?;

    query_client
        .params(QueryParamsRequest::default())
        .await?
        .into_inner()
        .params
        .context("staking params are empty")?
        .unbonding_time
        .context("missing unbonding period in staking params")
}

async fn get_latest_header<T>(light_client: &LightClient<T>) -> Result<Header>
where
    T: TendermintClient,
{
    let light_block = light_client.verify_to_highest().await?;
    Ok(light_block.signed_header.header)
}

fn get_block_height(chain_state: &ChainState, header: &Header) -> Height {
    let revision_number = chain_state.id.version();
    let revision_height = header.height.value();

    Height {
        revision_number,
        revision_height,
    }
}

#[cfg(feature = "wasm")]
async fn get_staking_query_client(grpc_addr: Url) -> Result<StakingQueryClient<Client>> {
    let mut url = grpc_addr.to_string();

    if url.ends_with('/') {
        url.pop();
    }

    let grpc_client = Client::new(url);
    Ok(StakingQueryClient::new(grpc_client))
}

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
async fn get_staking_query_client(grpc_addr: Url) -> Result<StakingQueryClient<Channel>> {
    StakingQueryClient::connect(grpc_addr.to_string())
        .await
        .context("error when initializing grpc client")
}
