mod chain;
mod home;
mod ibc;
mod not_found;
mod signer;

use stag_api::types::ics::core::ics24_host::identifier::ChainId;
use yew::prelude::*;
use yew_router::prelude::*;

use self::{
    chain::{AddChain, Chain, GetBalance, History},
    home::Home,
    ibc::{Burn, Connect, Ibc, Mint},
    not_found::NotFound,
    signer::{ImportSigner, Signer},
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/stag")]
    Home,
    #[at("/stag/signer")]
    Signer,
    #[at("/stag/signer/:s")]
    SignerNested,
    #[at("/stag/chain")]
    Chain,
    #[at("/stag/chain/:s")]
    ChainNested,
    #[at("/stag/ibc")]
    Ibc,
    #[at("/stag/ibc/:s")]
    IbcNested,
    #[at("/stag/history/:chain_id")]
    History { chain_id: ChainId },
    #[not_found]
    #[at("/stag/404")]
    NotFound,
}

#[derive(Clone, Routable, PartialEq)]
pub enum SignerRoute {
    #[at("/stag/signer/import")]
    Import,
    #[not_found]
    #[at("/stag/signer/404")]
    NotFound,
}

#[derive(Clone, Routable, PartialEq)]
pub enum ChainRoute {
    #[at("/stag/chain/add")]
    Add,
    #[at("/stag/chain/get-balance")]
    GetBalance,
    #[at("/stag/chain/history")]
    History,
    #[not_found]
    #[at("/stag/chain/404")]
    NotFound,
}

#[derive(Clone, Routable, PartialEq)]
pub enum IbcRoute {
    #[at("/stag/ibc/connect")]
    Connect,
    #[at("/stag/ibc/mint")]
    Mint,
    #[at("/stag/ibc/burn")]
    Burn,
    #[not_found]
    #[at("/stag/ibc/404")]
    NotFound,
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Signer => html! { <Signer /> },
        Route::SignerNested => html! {
            <Switch<SignerRoute> render={Switch::render(switch_signer)} />
        },
        Route::Chain => html! { <Chain /> },
        Route::ChainNested => html! {
            <Switch<ChainRoute> render={Switch::render(switch_chain)} />
        },
        Route::Ibc => html! { <Ibc /> },
        Route::IbcNested => html! {
            <Switch<IbcRoute> render={Switch::render(switch_ibc)} />
        },
        Route::History { chain_id } => html! { <History chain_id = { chain_id.clone() } /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

fn switch_signer(routes: &SignerRoute) -> Html {
    match routes {
        SignerRoute::Import => html! { <ImportSigner /> },
        SignerRoute::NotFound => html! {
            <Redirect<Route> to={Route::NotFound}/>
        },
    }
}

fn switch_chain(routes: &ChainRoute) -> Html {
    match routes {
        ChainRoute::Add => html! { <AddChain /> },
        ChainRoute::GetBalance => html! { <GetBalance /> },
        ChainRoute::History => html! { <History /> },
        ChainRoute::NotFound => html! {
            <Redirect<Route> to={Route::NotFound}/>
        },
    }
}

fn switch_ibc(routes: &IbcRoute) -> Html {
    match routes {
        IbcRoute::Connect => html! { <Connect /> },
        IbcRoute::Mint => html! { <Mint /> },
        IbcRoute::Burn => html! { <Burn /> },
        IbcRoute::NotFound => html! {
            <Redirect<Route> to={Route::NotFound}/>
        },
    }
}
