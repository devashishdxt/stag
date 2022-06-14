mod components;
mod routes;
mod store;

use yew::{html, Component, Context, Html};
use yew_router::{BrowserRouter, Switch};

use self::{
    components::notification::Notification,
    routes::{switch, Route},
};

fn main() {
    yew::start_app::<App>();
}

struct App;

impl Component for App {
    type Message = ();

    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <Notification />
                <Switch<Route> render={Switch::render(switch)} />
            </BrowserRouter>
        }
    }
}
