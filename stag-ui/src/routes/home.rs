use yew::prelude::*;

use crate::{
    components::{Navbar, Notification},
    routes::Route,
};

#[function_component(Home)]
pub fn home() -> Html {
    html! {
        <>
            <Navbar active = { Route::Home } />
            <Notification />
        </>
    }
}
