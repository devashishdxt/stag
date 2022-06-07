mod core;
mod home;
mod ica;
mod not_found;
mod page;
mod query;
mod transfer;

use yew::{html, Html};
use yew_router::Routable;

use crate::AppState;

use self::{
    core::{chains::Chains, channels::Channels, connections::Connections, signers::Signers},
    home::Home,
    ica::{bank::Bank, staking::Staking},
    not_found::NotFound,
    query::{balance::Balance, history::History, ica::Ica},
    transfer::{burn::Burn, mint::Mint},
};

#[derive(Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/core/signers")]
    Signers,
    #[at("/core/chains")]
    Chains,
    #[at("/core/connections")]
    Connections,
    #[at("/core/channels")]
    Channels,
    #[at("/transfer/mint")]
    Mint,
    #[at("/transfer/burn")]
    Burn,
    #[at("/ica/bank")]
    Bank,
    #[at("/ica/staking")]
    Staking,
    #[at("/query/ica")]
    Ica,
    #[at("/query/balance")]
    Balance,
    #[at("/query/history")]
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
            <Signers notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
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
            <Mint notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Burn => html! {
            <Burn notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Bank => html! {
            <Bank notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Staking => html! {
            <Staking notification={state.notification} signer={state.signer} storage={state.storage} rpc_client={state.rpc} event_handler={state.event_handler} />
        },
        Route::Ica => html! {
            <Ica notification={state.notification} signer={state.signer} storage={state.storage} />
        },
        Route::Balance => html! {
            <Balance notification={state.notification} signer={state.signer} storage={state.storage} />
        },
        Route::History => html! {
            <History notification={state.notification} storage={state.storage} />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
