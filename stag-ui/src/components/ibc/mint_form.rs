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
struct MintState {
    chain_id: UseStateHandle<String>,
    amount: UseStateHandle<String>,
    denom: UseStateHandle<String>,
    receiver: UseStateHandle<String>,
    memo: UseStateHandle<String>,
}

impl MintState {
    async fn mint<S>(
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
        let receiver: Option<String> = if self.receiver.is_empty() {
            None
        } else {
            Some((*self.receiver).clone())
        };

        let stag = Stag::builder()
            .with_signer(signer)?
            .with_rpc_client(rpc_client)
            .with_storage(storage)
            .await?
            .with_event_handler(event_handler)
            .build();

        stag.mint(
            chain_id,
            None,
            amount,
            denom,
            receiver,
            (*self.memo).clone(),
        )
        .await
    }
}

impl Default for MintState {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            amount: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
            receiver: use_state(|| "".to_string()),
            memo: use_state(|| "".to_string()),
        }
    }
}

#[function_component(MintForm)]
pub fn mint_form() -> Html {
    let state = MintState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <TextInput label="Amount" placeholder="Amount of tokens to mint" value={ state.amount.clone() }/>
            <TextInput label="Denom" placeholder="Denom of tokens to mint" value={ state.denom.clone() }/>
            <TextInput label="Receiver" placeholder="Receiver address on IBC enabled chain (if this is not provided, tokens will be sent to signer's address)" value={ state.receiver.clone() }/>
            <TextInput label="Memo" placeholder="Memo to send in IBC transaction (optional)" value={ state.memo.clone() }/>
            <button onclick = {
                move |_| {
                    let state = state.clone();
                    let history = history.clone();
                    let atom = atom.clone();
                    let notification = notification.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Minting tokens".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    wasm_bindgen_futures::spawn_local(async move {
                        match state.mint((*atom).signer.clone(), (*atom).storage.clone(), atom.rpc_client, atom.event_handler).await {
                            Ok(_) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: "Successfully minted tokens on chain".to_string(),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });
                                history.push(Route::Home);
                            },
                            Err(err) => {
                                notification.set(NotificationAtom {
                                    data: Some(NotificationData {
                                        message: format!("Failed to mint tokens on chain: {:?}", err),
                                        icon: NotificationIcon::Success,
                                        dismissable: true,
                                    })
                                });
                            },
                        }
                    });
                }
            }>{ "Mint" }</button>
        </form>
    }
}
