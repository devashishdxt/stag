use yew::{classes, function_component, html};
use yew_router::{hooks::use_route, prelude::Link};

use crate::routes::Route;

const CONTAINER_CLASSES: &[&str] = &[
    "fixed",
    "w-60",
    "bg-slate-900",
    "h-screen",
    "py-6",
    "text-slate-50",
];

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
        <div class={classes!(CONTAINER_CLASSES)}>
            <Link<Route> to={Route::Home}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::Home).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-house", "w-8", "text-center", "mr-4")}></i><span>{ "Home" }</span>
                </div>
            </Link<Route>>
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
            <Link<Route> to={Route::History}>
                <div class={classes!(LINK_CLASSES, (current_route == Route::History).then(|| ["border-r-4", "bg-slate-800"].as_ref()))}>
                    <i class={classes!("fa-solid", "fa-calendar-days", "w-8", "text-center", "mr-4")}></i><span>{ "History" }</span>
                </div>
            </Link<Route>>
        </div>
    }
}
