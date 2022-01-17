use anyhow::{Context, Result};
use bounce::use_atom;
use stag_api::{
    event::TracingEventHandler,
    signer::SignerConfig,
    stag::Stag,
    storage::IndexedDb,
    types::ics::core::ics24_host::identifier::{ChainId, Identifier},
};
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon, StagAtom},
    components::TextInput,
    routes::Route,
};

#[derive(Clone, PartialEq)]
struct GetBalanceState {
    chain_id: UseStateHandle<String>,
    denom: UseStateHandle<String>,
}

impl GetBalanceState {
    async fn get_balance<S>(
        &self,
        signer: S,
        storage: IndexedDb,
        event_handler: TracingEventHandler,
    ) -> Result<String>
    where
        S: SignerConfig,
    {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain ID")?;
        let denom: Identifier = self.denom.parse().context("Invalid denom")?;

        let stag = Stag::builder()
            .with_signer(signer)?
            .with_storage(storage)
            .await?
            .with_event_handler(event_handler)
            .build();

        stag.get_balance(&chain_id, &denom)
            .await
            .map(|balance| balance.to_string())
    }
}

impl Default for GetBalanceState {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
        }
    }
}

#[function_component(GetBalanceForm)]
pub fn get_balance_form() -> Html {
    let state = GetBalanceState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <TextInput label="Denom" placeholder="Denom of tokens" value={ state.denom.clone() }/>
            <button type="submit" onclick = {
                move |_| {
                    let state = state.clone();
                    let history = history.clone();
                    let atom = atom.clone();
                    let notification = notification.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Getting balance".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    wasm_bindgen_futures::spawn_local(async move {
                        match state.get_balance((*atom).signer.clone(), (*atom).storage.clone(), atom.event_handler).await {
                            Ok(balance) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Current Balance: {} {}", balance, *state.denom),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });
                                history.push(Route::Home);
                            },
                            Err(err) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Failed to get balance on chain: {:?}", err),
                                        icon: NotificationIcon::Error,
                                        dismissable: true,
                                    })
                                });
                            },
                        }
                    });
                }
            }>{ "Get Balance" }</button>
        </form>
    }
}
