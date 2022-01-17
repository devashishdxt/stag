use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{ChainList, Navbar, Notification},
    routes::{IbcRoute, Route},
};

#[function_component(Ibc)]
pub fn ibc() -> Html {
    html! {
        <>
            <Navbar active = { Route::Ibc } />
            <section>
                <ChainList/>
            </section>
            <section>
                <nav class={ "inner" }>
                    <Link<IbcRoute> to={ IbcRoute::Connect }>{ "Connect Chain" }</Link<IbcRoute>>
                    <span class="gap" />
                    <Link<IbcRoute> to={ IbcRoute::Mint }>{ "Mint Tokens" }</Link<IbcRoute>>
                    <span class="gap" />
                    <Link<IbcRoute> to={ IbcRoute::Burn }>{ "Burn Tokens" }</Link<IbcRoute>>
                </nav>
            </section>
            <Notification />
        </>
    }
}
