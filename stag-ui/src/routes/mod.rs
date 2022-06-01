mod burn;
mod chains;
mod channels;
mod connections;
mod history;
mod home;
mod mint;
mod not_found;
mod page;
mod signers;

use yew::{html, Html};
use yew_router::Routable;

use crate::AppState;

use self::{
    burn::Burn, chains::Chains, channels::Channels, connections::Connections, history::History,
    home::Home, mint::Mint, not_found::NotFound, signers::Signers,
};

#[derive(Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/signers")]
    Signers,
    #[at("/chains")]
    Chains,
    #[at("/connections")]
    Connections,
    #[at("/channels")]
    Channels,
    #[at("/mint")]
    Mint,
    #[at("/burn")]
    Burn,
    #[at("/history")]
    History,
    #[not_found]
    #[at("/not-found")]
    NotFound,
}

pub fn switch(route: &Route, state: AppState) -> Html {
    match route {
        Route::Home => html! {
            <Home />
        },
        Route::Signers => html! {
            <Signers notification={state.notification} signer={state.signer} />
        },
        Route::Chains => html! {
            <Chains notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Connections => html! {
            <Connections notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Channels => html! {
            <Channels notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Mint => html! {
            <Mint />
        },
        Route::Burn => html! {
            <Burn />
        },
        Route::History => html! {
            <History />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
