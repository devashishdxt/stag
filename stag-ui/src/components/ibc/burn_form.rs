use anyhow::{Context, Result};
use bounce::use_atom;
use primitive_types::U256;
use stag_api::{
    event::TracingEventHandler,
    signer::SignerConfig,
    stag::Stag,
    storage::IndexedDb,
    tendermint::ReqwestClient,
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
struct BurnState {
    chain_id: UseStateHandle<String>,
    amount: UseStateHandle<String>,
    denom: UseStateHandle<String>,
    memo: UseStateHandle<String>,
}

impl BurnState {
    async fn burn<S>(
        &self,
        signer: S,
        storage: IndexedDb,
        rpc_client: ReqwestClient,
        event_handler: TracingEventHandler,
    ) -> Result<String>
    where
        S: SignerConfig,
    {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain ID")?;
        let amount: U256 = U256::from_dec_str(&self.amount).context("Invalid amount")?;
        let denom: Identifier = self.denom.parse().context("Invalid denom")?;

        let stag = Stag::builder()
            .with_signer(signer)?
            .with_rpc_client(rpc_client)
            .with_storage(storage)
            .await?
            .with_event_handler(event_handler)
            .build();

        stag.burn(chain_id, None, amount, denom, (*self.memo).clone())
            .await
    }
}

impl Default for BurnState {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            amount: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
            memo: use_state(|| "".to_string()),
        }
    }
}

#[function_component(BurnForm)]
pub fn burn_form() -> Html {
    let state = BurnState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <TextInput label="Amount" placeholder="Amount of tokens to burn" value={ state.amount.clone() }/>
            <TextInput label="Denom" placeholder="Denom of tokens to burn" value={ state.denom.clone() }/>
            <TextInput label="Memo" placeholder="Memo to send in IBC transaction (optional)" value={ state.memo.clone() }/>
            <button onclick = {
                move |_| {
                    let state = state.clone();
                    let history = history.clone();
                    let atom = atom.clone();
                    let notification = notification.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Burning tokens".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    wasm_bindgen_futures::spawn_local(async move {
                        match state.burn((*atom).signer.clone(), (*atom).storage.clone(), atom.rpc_client, atom.event_handler).await {
                            Ok(_) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: "Successfully burnt tokens on chain".to_string(),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });
                                history.push(Route::Home);
                            },
                            Err(err) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Failed to burn tokens on chain: {:?}", err),
                                        icon: NotificationIcon::Error,
                                        dismissable: true,
                                    })
                                });
                            },
                        }
                    });
                }
            }>{ "Burn" }</button>
        </form>
    }
}
