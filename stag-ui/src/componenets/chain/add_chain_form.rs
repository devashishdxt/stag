use anyhow::{ensure, Context, Result};
use humantime::parse_duration;
use stag_api::{
    event::TracingEventHandler,
    signer::MnemonicSigner,
    stag::Stag,
    storage::IndexedDb,
    tendermint::ReqwestClient,
    types::chain_state::{ChainConfig, Fee},
};
use tracing::error;
use web_sys::FocusEvent;
use yew::{classes, function_component, html, use_state, Callback, Properties, UseStateHandle};

use crate::componenets::{notification::NotificationData, text_input::TextInput};

const BUTTON_CLASSES: &[&str] = &[
    "px-8",
    "py-2",
    "rounded",
    "bg-slate-200",
    "hover:bg-slate-300",
    "hover:shadow",
    "transition-all",
];

#[derive(Clone)]
struct State {
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
    trusted_height: UseStateHandle<String>,
    trusted_hash: UseStateHandle<String>,
    packet_timeout_height_offset: UseStateHandle<String>,
}

impl State {
    #[allow(clippy::type_complexity)]
    fn parse(&self) -> Result<ChainConfig> {
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
        self.trusted_height.set("".to_string());
        self.trusted_hash.set("".to_string());
        self.packet_timeout_height_offset.set("20".to_string());
    }

    fn clear(&self) {
        self.grpc_addr.set("".to_string());
        self.rpc_addr.set("".to_string());
        self.fee_amount.set("".to_string());
        self.fee_denom.set("".to_string());
        self.gas_limit.set("".to_string());
        self.trust_level.set("".to_string());
        self.trusting_period.set("".to_string());
        self.max_clock_drift.set("".to_string());
        self.rpc_timeout.set("".to_string());
        self.diversifier.set("".to_string());
        self.trusted_height.set("".to_string());
        self.trusted_hash.set("".to_string());
        self.packet_timeout_height_offset.set("".to_string());
    }
}

impl Default for State {
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
            trusted_height: use_state(|| "".to_string()),
            trusted_hash: use_state(|| "".to_string()),
            packet_timeout_height_offset: use_state(|| "".to_string()),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
    pub event_handler: TracingEventHandler,
}

#[function_component(AddChainForm)]
pub fn add_chain_form(props: &Props) -> Html {
    let state = State::default();

    let on_submit = Callback::from({
        let notification = props.notification.clone();
        let signer = props.signer.clone();
        let storage = props.storage.clone();
        let rpc_client = props.rpc_client;
        let event_handler = props.event_handler;

        let state = state.clone();

        move |event: FocusEvent| {
            event.prevent_default();

            let notification = notification.clone();
            let signer = signer.clone();
            let storage = storage.clone();

            let state = state.clone();

            notification.set(Some(NotificationData::processing(
                "Adding chain".to_string(),
            )));

            match state.parse() {
                Ok(chain_config) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match add_chain(
                            (*signer).clone(),
                            storage,
                            rpc_client,
                            event_handler,
                            chain_config,
                        )
                        .await
                        {
                            Ok(chain_id) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(format!(
                                    "Added chain: {}",
                                    chain_id
                                ))));
                            }
                            Err(err) => {
                                error!("Failed to add chain: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to add chain".to_string(),
                                )));
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("Invalid data: {:?}", err);
                    notification.set(Some(NotificationData::error(err.to_string())));
                }
            }
        }
    });

    html! {
        <div class={classes!("border-t-2", "border-slate-400", "p-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Add Chain" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="gRPC address" value={ state.grpc_addr.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="RPC address" value={ state.rpc_addr.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Fee amount" value={ state.fee_amount.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Fee denom" value={ state.fee_denom.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Gas limit" value={ state.gas_limit.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Trust level (e.g. '1/3')" value={ state.trust_level.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Trusting period: duration since the LastestTimestamp during which the submitted headers are valid for upgrade (e.g. '14 days')" value={ state.trusting_period.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Max clock drift: defines how much new (untrusted) header's time can drift into the future (e.g. '3 sec')" value={ state.max_clock_drift.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Light client RPC timeout (e.g. '60 sec')" value={ state.rpc_timeout.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Diversifier for solo machine" value={ state.diversifier.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Trusted height of chain for light client" value={ state.trusted_height.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Trusted hash of chain for light client" value={ state.trusted_hash.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Packet timout height offset: block height after which the packet times out" value={ state.packet_timeout_height_offset.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    let state = state.clone();
                    move |_| state.fill_defaults()
                }>{ "Fill Defaults" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}

async fn add_chain(
    signer: MnemonicSigner,
    storage: IndexedDb,
    rpc_client: ReqwestClient,
    event_handler: TracingEventHandler,
    chain_config: ChainConfig,
) -> Result<String> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(rpc_client)
        .with_event_handler(event_handler)
        .build();

    let chain_id = stag.add_chain(&chain_config).await?;

    Ok(chain_id.to_string())
}

fn parse_trusted_hash(hash: &str) -> Result<[u8; 32]> {
    ensure!(!hash.is_empty(), "empty trusted hash");

    let bytes = hex::decode(hash).context("invalid trusted hash hex bytes")?;
    ensure!(bytes.len() == 32, "trusted hash length should be 32");

    let mut trusted_hash = [0; 32];
    trusted_hash.clone_from_slice(&bytes);

    Ok(trusted_hash)
}
