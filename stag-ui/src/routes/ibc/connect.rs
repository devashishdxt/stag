use yew::prelude::*;

use crate::{
    components::{ConnectForm, Navbar, Notification},
    routes::Route,
};

#[function_component(Connect)]
pub fn connect() -> Html {
    html! {
        <>
            <Navbar active = { Route::Ibc } />
            <ConnectForm />
            <Notification />
        </>
    }
}
