use anyhow::{Context, Result};
use stag_api::{
    signer::MnemonicSigner,
    stag::Stag,
    storage::IndexedDb,
    types::ics::core::ics24_host::identifier::{ChainId, Identifier, PortId},
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
    denom: UseStateHandle<String>,
    is_ibc_denom: UseStateHandle<bool>,
}

impl State {
    fn parse(&self) -> Result<(ChainId, Identifier, bool)> {
        let chain_id = (*self.chain_id).parse().context("Invalid chain ID")?;
        let denom = (*self.denom).parse().context("Invalid denom")?;
        let is_ibc_denom = *self.is_ibc_denom;

        Ok((chain_id, denom, is_ibc_denom))
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
        self.denom.set("".to_string());
        self.is_ibc_denom.set(true);
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
            is_ibc_denom: use_state(|| true),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
}

#[function_component(GetBalanceForm)]
pub fn get_balance_form(props: &Props) -> Html {
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
                "Fetching balance".to_string(),
            )));

            match state.parse() {
                Ok((chain_id, denom, is_ibc_denom)) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match get_balance(
                            (*signer).clone(),
                            storage,
                            chain_id,
                            denom.clone(),
                            is_ibc_denom,
                        )
                        .await
                        {
                            Ok(balance) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(format!(
                                    "Current Balance: {} {}",
                                    balance, denom
                                ))));
                            }
                            Err(err) => {
                                error!("Failed to fetch balance: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to fetch balance".to_string(),
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
        <div class={classes!("p-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Get Balance" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Denom" value={ state.denom.clone() } />
                <CheckboxInput class={classes!("mb-4")} placeholder="Is IBC denom?" value={ state.is_ibc_denom.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}

async fn get_balance(
    signer: MnemonicSigner,
    storage: IndexedDb,
    chain_id: ChainId,
    denom: Identifier,
    is_ibc_denom: bool,
) -> Result<String> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .build();

    if is_ibc_denom {
        stag.get_ibc_balance(&chain_id, &PortId::transfer(), &denom)
            .await
            .map(|balance| balance.to_string())
    } else {
        stag.get_balance(&chain_id, &denom)
            .await
            .map(|balance| balance.to_string())
    }
}
