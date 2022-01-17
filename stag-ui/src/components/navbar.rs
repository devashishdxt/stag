use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[derive(Clone, PartialEq, Properties)]
pub struct NavbarProps {
    pub active: Route,
}

#[function_component(Navbar)]
pub fn navbar(props: &NavbarProps) -> Html {
    html! {
        <nav class={ "top" }>
            <Link<Route> to={Route::Home} classes = { classes!((props.active == Route::Home).then(|| "active")) }>{ "Home" }</Link<Route>>
            <Link<Route> to={Route::Signer} classes = { classes!((props.active == Route::Signer).then(|| "active")) }>{ "Signer" }</Link<Route>>
            <Link<Route> to={Route::Chain} classes = { classes!((props.active == Route::Chain).then(|| "active")) }>{ "Chain" }</Link<Route>>
            <Link<Route> to={Route::Ibc} classes = { classes!((props.active == Route::Ibc).then(|| "active")) }>{ "IBC" }</Link<Route>>
        </nav>
    }
}
