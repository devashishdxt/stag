use yew::prelude::*;

use crate::{
    components::{GetBalanceForm, Navbar, Notification},
    routes::Route,
};

#[function_component(GetBalance)]
pub fn get_balance() -> Html {
    html! {
        <>
            <Navbar active = { Route::Chain } />
            <GetBalanceForm />
            <Notification />
        </>
    }
}
