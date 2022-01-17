use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{ChainList, Navbar, Notification},
    routes::{ChainRoute, Route},
};

#[function_component(Chain)]
pub fn chain() -> Html {
    html! {
        <>
            <Navbar active = { Route::Chain } />
            <section>
                <ChainList/>
            </section>
            <section>
                <nav class={ "inner" }>
                    <Link<ChainRoute> to={ ChainRoute::Add }>{ "Add" }</Link<ChainRoute>>
                    <span class="gap" />
                    <Link<ChainRoute> to={ ChainRoute::GetBalance }>{ "Get Balance" }</Link<ChainRoute>>
                    <span class="gap" />
                    <Link<ChainRoute> to={ ChainRoute::History }>{ "Get History" }</Link<ChainRoute>>
                </nav>
            </section>
            <Notification />
        </>
    }
}
