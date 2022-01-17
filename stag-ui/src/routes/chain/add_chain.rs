use yew::prelude::*;

use crate::{
    components::{AddChainForm, Navbar, Notification},
    routes::Route,
};

#[function_component(AddChain)]
pub fn add_chain() -> Html {
    html! {
        <>
            <Navbar active = { Route::Chain } />
            <AddChainForm />
            <Notification />
        </>
    }
}
