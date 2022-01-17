use anyhow::{ensure, Context, Result};
use bounce::use_atom;
use humantime::parse_duration;
use stag_api::{
    signer::SignerConfig,
    stag::Stag,
    storage::IndexedDb,
    tendermint::ReqwestClient,
    types::{
        chain_state::{ChainConfig, Fee},
        ics::core::ics24_host::identifier::ChainId,
    },
};
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon, StagAtom},
    components::TextInput,
    routes::Route,
};

#[derive(Clone)]
struct AddChainState {
    grpc_addr: UseStateHandle<String>,
    rpc_addr: UseStateHandle<String>,
    fee_amount: UseStateHandle<String>,
    fee_denom: UseStateHandle<String>,
    gas_limit: UseStateHandle<String>,
    trust_level: UseStateHandle<String>,
    trusting_period: UseStateHandle<String>,
    max_clock_drift: UseStateHandle<String>,
    rpc_timeout: UseStateHandle<String>,
    diversifier: UseStateHandle<String>,
    port_id: UseStateHandle<String>,
    trusted_height: UseStateHandle<String>,
    trusted_hash: UseStateHandle<String>,
    packet_timeout_height_offset: UseStateHandle<String>,
}

impl AddChainState {
    fn fill_defaults(&self) {
        self.grpc_addr.set("http://localhost:9091/".to_string());
        self.rpc_addr.set("http://localhost:26657/".to_string());
        self.fee_amount.set("1000".to_string());
        self.fee_denom.set("stake".to_string());
        self.gas_limit.set("300000".to_string());
        self.trust_level.set("1/3".to_string());
        self.trusting_period.set("14 days".to_string());
        self.max_clock_drift.set("3 sec".to_string());
        self.rpc_timeout.set("60 sec".to_string());
        self.diversifier.set("stag".to_string());
        self.port_id.set("transfer".to_string());
        self.trusted_height.set("".to_string());
        self.trusted_hash.set("".to_string());
        self.packet_timeout_height_offset.set("20".to_string());
    }

    fn get_chain_config(&self) -> Result<ChainConfig> {
        Ok(ChainConfig {
            grpc_addr: self.grpc_addr.parse().context("Invalid gRPC address")?,
            rpc_addr: self.rpc_addr.parse().context("Invalid RPC address")?,
            fee: Fee {
                amount: self.fee_amount.parse().context("Invalid fee amount")?,
                denom: self.fee_denom.parse().context("Invalid fee denom")?,
                gas_limit: self.gas_limit.parse().context("Invalid gas limit")?,
            },
            trust_level: self.trust_level.parse().context("Invalid trust level")?,
            trusting_period: parse_duration(&self.trusting_period)
                .context("Invalid trusting period")?,
            max_clock_drift: parse_duration(&self.max_clock_drift)
                .context("Invalid max clock drift")?,
            rpc_timeout: parse_duration(&self.rpc_timeout).context("Invalid RPC timeout")?,
            diversifier: (*self.diversifier).clone(),
            port_id: self.port_id.parse().context("Invalid port ID")?,
            trusted_height: self
                .trusted_height
                .parse()
                .context("Invalid trusted height")?,
            trusted_hash: parse_trusted_hash(&self.trusted_hash).context("Invalid trusted hash")?,
            packet_timeout_height_offset: self
                .packet_timeout_height_offset
                .parse()
                .context("Invalid packet timeout height offset")?,
        })
    }

    async fn add_chain<S>(
        &self,
        signer: S,
        storage: IndexedDb,
        rpc_client: ReqwestClient,
    ) -> Result<ChainId>
    where
        S: SignerConfig,
    {
        let chain_config = self.get_chain_config()?;

        let stag = Stag::builder()
            .with_signer(signer)?
            .with_rpc_client(rpc_client)
            .with_storage(storage)
            .await?
            .build();

        stag.add_chain(&chain_config).await
    }
}

impl Default for AddChainState {
    fn default() -> Self {
        Self {
            grpc_addr: use_state(|| "".to_string()),
            rpc_addr: use_state(|| "".to_string()),
            fee_amount: use_state(|| "".to_string()),
            fee_denom: use_state(|| "".to_string()),
            gas_limit: use_state(|| "".to_string()),
            trust_level: use_state(|| "".to_string()),
            trusting_period: use_state(|| "".to_string()),
            max_clock_drift: use_state(|| "".to_string()),
            rpc_timeout: use_state(|| "".to_string()),
            diversifier: use_state(|| "".to_string()),
            port_id: use_state(|| "".to_string()),
            trusted_height: use_state(|| "".to_string()),
            trusted_hash: use_state(|| "".to_string()),
            packet_timeout_height_offset: use_state(|| "".to_string()),
        }
    }
}

#[function_component(AddChainForm)]
pub fn add_chain_form() -> Html {
    let state = AddChainState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <button type="button" onclick = {
                let state = state.clone();
                move |_| state.fill_defaults()
            }>{ "Fill Defaults" }</button>
            <TextInput label="gRPC Address" placeholder="gRPC address of IBC enabled chain" value={ state.grpc_addr.clone() }/>
            <TextInput label="RPC Address" placeholder="RPC address of IBC enabled chain" value={ state.rpc_addr.clone() }/>
            <TextInput label="Fee Amount" placeholder="Fee amount to be used in each cosmos sdk transaction" value={ state.fee_amount.clone() }/>
            <TextInput label="Fee Denom" placeholder="Fee denom to be used in each cosmos sdk transaction" value={ state.fee_denom.clone() }/>
            <TextInput label="Gas Limit" placeholder="Gas limit to be used in each cosmos sdk transaction" value={ state.gas_limit.clone() }/>
            <TextInput label="Trust Level" placeholder="Trust level of IBC enabled chain (e.g. 1/3)" value={ state.trust_level.clone() }/>
            <TextInput label="Trusting Period" placeholder="Duration of the period since the LastestTimestamp during which the submitted headers are valid for upgrade (e.g. 14 days)" value={ state.trusting_period.clone() }/>
            <TextInput label="Max Clock Drift" placeholder="Defines how much new (untrusted) header's time can drift into the future (e.g. 3 sec)" value={ state.max_clock_drift.clone() }/>
            <TextInput label="RPC Timeout" placeholder="Light client RPC timeout (e.g. 60 sec)" value={ state.rpc_timeout.clone() }/>
            <TextInput label="Diversifier" placeholder="Diversifier for solo machine" value={ state.diversifier.clone() }/>
            <TextInput label="Port ID" placeholder="Port ID of IBC channel" value={ state.port_id.clone() }/>
            <TextInput label="Trusted Height" placeholder="Trusted height of chain for light client" value={ state.trusted_height.clone() }/>
            <TextInput label="Trusted Hash" placeholder="Trusted hash of chain for light client" value={ state.trusted_hash.clone() }/>
            <TextInput label="Packet Timeout Height Offset" placeholder="Block height after which the packet times out" value={ state.packet_timeout_height_offset.clone() }/>
            <button type="submit" onclick = {
                move |_| {
                    let state = state.clone();
                    let history = history.clone();
                    let atom = atom.clone();
                    let notification = notification.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Adding chain".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    wasm_bindgen_futures::spawn_local(async move {
                        match state.add_chain((*atom).signer.clone(), (*atom).storage.clone(), atom.rpc_client).await {
                            Ok(chain_id) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Added chain {}", chain_id),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });

                                history.push(Route::Home);
                            },
                            Err(err) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Failed to add chain: {:?}", err),
                                        icon: NotificationIcon::Error,
                                        dismissable: true,
                                    })
                                });
                            },
                        }
                    });
                }
            }>{ "Add Chain" }</button>
        </form>
    }
}

fn parse_trusted_hash(hash: &str) -> Result<[u8; 32]> {
    ensure!(!hash.is_empty(), "empty trusted hash");

    let bytes = hex::decode(hash).context("invalid trusted hash hex bytes")?;
    ensure!(bytes.len() == 32, "trusted hash length should be 32");

    let mut trusted_hash = [0; 32];
    trusted_hash.clone_from_slice(&bytes);

    Ok(trusted_hash)
}
