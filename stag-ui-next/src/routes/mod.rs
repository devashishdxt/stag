mod core;
mod home;
mod not_found;
mod page;

use yew::{html, Html};
use yew_router::Routable;

use self::{core::signers::Signers, home::Home, not_found::NotFound};

#[derive(Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/core/signers")]
    Signers,
    #[not_found]
    #[at("/not-found")]
    NotFound,
}

pub fn switch(routes: &Route) -> Html {
    match routes {
        Route::Home => html! {
            <Home />
        },
        Route::Signers => html! {
            <Signers />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
