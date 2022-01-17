use anyhow::{Context, Result};
use bounce::use_atom;
use stag_api::{
    signer::SignerConfig, stag::Stag, storage::IndexedDb, tendermint::ReqwestClient,
    types::ics::core::ics24_host::identifier::ChainId,
};
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon, StagAtom},
    components::{CheckboxInput, TextInput},
    routes::Route,
};

#[derive(Clone, PartialEq)]
struct ConnectState {
    chain_id: UseStateHandle<String>,
    memo: UseStateHandle<String>,
    force: UseStateHandle<bool>,
}

impl ConnectState {
    async fn connect<S>(
        &self,
        signer: S,
        storage: IndexedDb,
        rpc_client: ReqwestClient,
    ) -> Result<()>
    where
        S: SignerConfig,
    {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain ID")?;

        let stag = Stag::builder()
            .with_signer(signer)?
            .with_rpc_client(rpc_client)
            .with_storage(storage)
            .await?
            .build();

        stag.connect(chain_id, None, (*self.memo).clone(), *self.force)
            .await
    }
}

impl Default for ConnectState {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            memo: use_state(|| "".to_string()),
            force: use_state(|| false),
        }
    }
}

#[function_component(ConnectForm)]
pub fn connect_form() -> Html {
    let state = ConnectState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <TextInput label="Memo" placeholder="Memo to send in IBC transaction (optional)" value={ state.memo.clone() }/>
            <CheckboxInput label="Force" value={ state.force.clone() }/>
            <button onclick = {
                move |_| {
                    let state = state.clone();
                    let history = history.clone();
                    let atom = atom.clone();
                    let notification = notification.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Establishing IBC connection with chain".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    wasm_bindgen_futures::spawn_local(async move {
                        match state.connect((*atom).signer.clone(), (*atom).storage.clone(), atom.rpc_client).await {
                            Ok(()) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: "Successfully established IBC connection".to_string(),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });
                                history.push(Route::Home);
                            },
                            Err(err) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Failed to establish IBC connection: {:?}", err),
                                        icon: NotificationIcon::Error,
                                        dismissable: true,
                                    })
                                });
                            },
                        }
                    });
                }
            }>{ "Connect" }</button>
        </form>
    }
}
