use anyhow::{Context, Result};
use stag_api::{
    signer::MnemonicSigner, stag::Stag, storage::IndexedDb,
    types::ics::core::ics24_host::identifier::ChainId,
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
    chain_id: UseStateHandle<String>,
}

impl State {
    fn parse(&self) -> Result<ChainId> {
        let chain_id = (*self.chain_id).parse().context("Invalid chain ID")?;

        Ok(chain_id)
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
}

#[function_component(GetIcaAddressForm)]
pub fn get_ica_address_form(props: &Props) -> Html {
    let state = State::default();

    let on_submit = Callback::from({
        let notification = props.notification.clone();
        let signer = props.signer.clone();
        let storage = props.storage.clone();

        let state = state.clone();

        move |event: FocusEvent| {
            event.prevent_default();

            let notification = notification.clone();
            let signer = signer.clone();
            let storage = storage.clone();

            let state = state.clone();

            notification.set(Some(NotificationData::processing(
                "Fetching ICA address".to_string(),
            )));

            match state.parse() {
                Ok(chain_id) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match get_ica_address((*signer).clone(), storage, chain_id).await {
                            Ok(ica_address) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(format!(
                                    "ICA address: {}",
                                    ica_address
                                ))));
                            }
                            Err(err) => {
                                error!("Failed to fetch ICA address: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to ICA address".to_string(),
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
        <div class={classes!("border-t-2", "border-slate-400", "p-6", "mt-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Get ICA Address" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}

async fn get_ica_address(
    signer: MnemonicSigner,
    storage: IndexedDb,
    chain_id: ChainId,
) -> Result<String> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .build();

    stag.get_ica_address(&chain_id).await
}
