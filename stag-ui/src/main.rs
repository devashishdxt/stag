#![allow(clippy::let_unit_value)] // TODO: migrate to yew 0.20
mod componenets;
mod routes;

use componenets::notification::NotificationData;
use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};
use yew::{classes, function_component, html, use_state, UseStateHandle};
use yew_router::{BrowserRouter, Switch};

use self::{
    componenets::{notification::Notification, sidebar::Sidebar},
    routes::{switch, Route},
};

pub struct AppState {
    notification: UseStateHandle<Option<NotificationData>>,
    signer: UseStateHandle<MnemonicSigner>,
    storage: IndexedDb,
    rpc: ReqwestClient,
    event_handler: TracingEventHandler,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            notification: self.notification.clone(),
            signer: self.signer.clone(),
            storage: self.storage.clone(),
            rpc: self.rpc,
            event_handler: self.event_handler,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        let notification = use_state(|| None);
        let signer = use_state(MnemonicSigner::default);
        let storage = IndexedDb::new("stag-ui");
        let rpc = ReqwestClient;
        let event_handler = TracingEventHandler;

        Self {
            notification,
            signer,
            storage,
            rpc,
            event_handler,
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let state = AppState::default();

    html! {
        <BrowserRouter>
            <Notification data={state.notification.clone()} />
            <div class={classes!("flex")}>
                <Sidebar />
                <div class={classes!("w-full")}>
                    <Switch<Route> render={Switch::render(move |route| switch(route, state.clone()))} />
                </div>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    tracing_wasm::set_as_global_default();
    yew::start_app::<App>();
}
