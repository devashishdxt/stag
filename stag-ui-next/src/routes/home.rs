use yew::{classes, html, Component, Context, Html};

pub struct Home;

impl Component for Home {
    type Message = ();

    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <>
                <h1 class={classes!("text-6xl", "font-bold", "text-center", "pt-40", "pb-10", "text-slate-900")}>{ "Welcome to Stag" }</h1>
                <h2 class={classes!("text-3xl", "font-bold", "text-center", "pb-20", "text-slate-900")}>{ "An IBC Solo Machine Implementation" }</h2>
            </>
        }
    }
}
