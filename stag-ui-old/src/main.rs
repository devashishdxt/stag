mod atoms;
mod components;
mod routes;

use bounce::BounceRoot;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::{switch, Route};

#[function_component(App)]
fn app() -> Html {
    html! {
        <BounceRoot>
            <BrowserRouter>
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        </BounceRoot>
    }
}

fn main() {
    tracing_wasm::set_as_global_default();

    yew::start_app::<App>();
}