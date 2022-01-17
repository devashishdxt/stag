use yew::prelude::*;

use crate::{
    components::{BurnForm, Navbar, Notification},
    routes::Route,
};

#[function_component(Burn)]
pub fn burn() -> Html {
    html! {
        <>
            <Navbar active = { Route::Ibc } />
            <BurnForm />
            <Notification />
        </>
    }
}
