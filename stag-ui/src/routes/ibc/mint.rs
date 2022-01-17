use yew::prelude::*;

use crate::{
    components::{MintForm, Navbar, Notification},
    routes::Route,
};

#[function_component(Mint)]
pub fn mint() -> Html {
    html! {
        <>
            <Navbar active = { Route::Ibc } />
            <MintForm />
            <Notification />
        </>
    }
}
