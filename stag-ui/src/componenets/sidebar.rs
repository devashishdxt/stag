use yew::{classes, function_component, html};
use yew_router::{hooks::use_route, prelude::Link};

use crate::routes::Route;

const CONTAINER_CLASSES: &[&str] = &["w-60", "min-w-max", "bg-slate-900", "py-6", "text-slate-50"];

const LINK_CLASSES: &[&str] = &[
    "p-4",
    "cursor-pointer",
    "hover:bg-slate-700",
    "hover:border-r-4",
    "transition-all",
];

#[function_component(Sidebar)]
pub fn sidebar() -> Html {
    let current_route: Route = use_route().unwrap_or_default();

    html! {
        <div class={classes!(CONTAINER_CLASSES, "min-h-screen")}>
            <Link<Route> to={Route::Home}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Home).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-house", "w-8", "text-center", "mr-4")}></i><span>{ "Home" }</span>
                </div>
            </Link<Route>>
            <fieldset class={classes!("border-t-2", "border-slate-700", "border-dotted", "my-2")}><legend class={classes!("text-sm", "text-slate-400", "mx-4", "px-2")}>{ "Core" }</legend></fieldset>
            <Link<Route> to={Route::Signers}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Signers).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-signature", "w-8", "text-center", "mr-4")}></i><span>{ "Signers" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Chains}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Chains).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-link", "w-8", "text-center", "mr-4")}></i><span>{ "Chains" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Connections}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Connections).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-plug", "w-8", "text-center", "mr-4")}></i><span>{ "Connections" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Channels}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Channels).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-bridge", "w-8", "text-center", "mr-4")}></i><span>{ "Channels" }</span>
                </div>
            </Link<Route>>
            <fieldset class={classes!("border-t-2", "border-slate-700", "border-dotted", "my-2")}><legend class={classes!("text-sm", "text-slate-400", "mx-4", "px-2")}>{ "Transfer" }</legend></fieldset>
            <Link<Route> to={Route::Mint}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Mint).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-coins", "w-8", "text-center", "mr-4")}></i><span>{ "Mint" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Burn}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Burn).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-fire", "w-8", "text-center", "mr-4")}></i><span>{ "Burn" }</span>
                </div>
            </Link<Route>>
            <fieldset class={classes!("border-t-2", "border-slate-700", "border-dotted", "my-2")}><legend class={classes!("text-sm", "text-slate-400", "mx-4", "px-2")}>{ "Interchain Accounts" }</legend></fieldset>
            <Link<Route> to={Route::Bank}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Bank).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-building-columns", "w-8", "text-center", "mr-4")}></i><span>{ "Bank" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Staking}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Staking).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-handshake-angle", "w-8", "text-center", "mr-4")}></i><span>{ "Staking" }</span>
                </div>
            </Link<Route>>
            <fieldset class={classes!("border-t-2", "border-slate-700", "border-dotted", "my-2")}><legend class={classes!("text-sm", "text-slate-400", "mx-4", "px-2")}>{ "Query" }</legend></fieldset>
            <Link<Route> to={Route::Ica}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Ica).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-user-astronaut", "w-8", "text-center", "mr-4")}></i><span>{ "ICA" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::Balance}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Balance).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-sack-dollar", "w-8", "text-center", "mr-4")}></i><span>{ "Balance" }</span>
                </div>
            </Link<Route>>
            <Link<Route> to={Route::History}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::History).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-calendar-days", "w-8", "text-center", "mr-4")}></i><span>{ "History" }</span>
                </div>
            </Link<Route>>
        </div>
    }
}
