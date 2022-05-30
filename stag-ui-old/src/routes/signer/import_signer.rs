use yew::prelude::*;

use crate::{
    components::{ImportSignerForm, Navbar, Notification},
    routes::Route,
};

#[function_component(ImportSigner)]
pub fn add_signer() -> Html {
    html! {
        <>
            <Navbar active = { Route::Signer } />
            <ImportSignerForm />
            <Notification />
        </>
    }
}
