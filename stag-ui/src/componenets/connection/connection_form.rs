use anyhow::{Context, Result};
use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, stag::Stag, storage::IndexedDb,
    tendermint::ReqwestClient, types::ics::core::ics24_host::identifier::ChainId,
};
use tracing::error;
use web_sys::FocusEvent;
use yew::{classes, function_component, html, use_state, Callback, Properties, UseStateHandle};

use crate::componenets::{
    checkbox_input::CheckboxInput, notification::NotificationData, text_input::TextInput,
};

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
    chain_id: UseStateHandle<String>,
    memo: UseStateHandle<String>,
    force: UseStateHandle<bool>,
}

impl State {
    #[allow(clippy::type_complexity)]
    fn parse(&self) -> Result<(ChainId, String, bool)> {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain id")?;
        let memo = (*self.memo).clone();
        let force = *self.force;

        Ok((chain_id, memo, force))
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
        self.memo.set("".to_string());
        self.force.set(false);
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            memo: use_state(|| "".to_string()),
            force: use_state(|| false),
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

#[function_component(ConnectionForm)]
pub fn connection_form(props: &Props) -> Html {
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
                "Connecting to chain".to_string(),
            )));

            match state.parse() {
                Ok((chain_id, memo, force)) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match connect(
                            (*signer).clone(),
                            storage,
                            rpc_client,
                            event_handler,
                            chain_id,
                            memo,
                            force,
                        )
                        .await
                        {
                            Ok(()) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(
                                    "Successfully connected to chain".to_string(),
                                )));
                            }
                            Err(err) => {
                                error!("Failed to connect to chain: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to connect to chain".to_string(),
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
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Connect Chain" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Memo to send in IBC transaction (optional)" value={ state.memo.clone() } />
                <CheckboxInput class={classes!("mb-4")} placeholder="Force" value={ state.force.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}

async fn connect(
    signer: MnemonicSigner,
    storage: IndexedDb,
    rpc_client: ReqwestClient,
    event_handler: TracingEventHandler,
    chain_id: ChainId,
    memo: String,
    force: bool,
) -> Result<()> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(rpc_client)
        .with_event_handler(event_handler)
        .build();

    stag.connect(chain_id, None, memo, force).await
}
