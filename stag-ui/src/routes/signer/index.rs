use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{Navbar, Notification, SignerList},
    routes::{Route, SignerRoute},
};

#[function_component(Signer)]
pub fn signer() -> Html {
    html! {
        <>
            <Navbar active = { Route::Signer } />
            <section>
                <SignerList/>
            </section>
            <section>
                <nav class={ "inner" }>
                    <Link<SignerRoute> to={ SignerRoute::Import }>{ "Import" }</Link<SignerRoute>>
                </nav>
            </section>
            <Notification />
        </>
    }
}
