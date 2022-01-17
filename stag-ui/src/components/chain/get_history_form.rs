use anyhow::{Context, Result};
use bounce::use_atom;
use stag_api::types::ics::core::ics24_host::identifier::ChainId;
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon},
    components::TextInput,
    routes::Route,
};

#[derive(Clone, PartialEq)]
struct GetHistoryState {
    chain_id: UseStateHandle<String>,
}

impl Default for GetHistoryState {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
        }
    }
}

#[function_component(GetHistoryForm)]
pub fn get_history_form() -> Html {
    let state = GetHistoryState::default();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <button type="submit" onclick = {
                move |_| {
                    let history = history.clone();
                    let chain_id: Result<ChainId> = state.chain_id.parse().context("Invalid chain id");

                    match chain_id {
                        Ok(chain_id) => history.push(Route::History { chain_id }),
                        Err(err) => notification.set(NotificationAtom {
                            data: Some(NotificationData {
                                message: format!("Failed to get transaction history: {:?}", err),
                                icon: NotificationIcon::Error,
                                dismissable: true,
                            })
                        }),
                    }
                }
            }>{ "Get History" }</button>
        </form>
    }
}
