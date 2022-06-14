use yew::{classes, html, Component, Context, Html};

pub struct NotFound;

impl Component for NotFound {
    type Message = ();

    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <p class={classes!("font-bold", "text-center", "pt-40", "pb-10", "text-slate-900")}>
                <i class={classes!("fa-solid", "fa-triangle-exclamation", "text-5xl")}></i>
                <h2 class={classes!("text-3xl", "pt-6", "text-slate-900")}>{ "Page not found!" }</h2>
            </p>
        }
    }
}
