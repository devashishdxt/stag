use yew::{classes, function_component, html};

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <p class={classes!("font-bold", "text-center", "pt-40", "pb-10", "text-slate-900")}>
            <i class={classes!("fa-solid", "fa-triangle-exclamation", "text-5xl")}></i>
            <h2 class={classes!("text-3xl", "pt-6", "text-slate-900")}>{ "Page not found!" }</h2>
        </p>
    }
}
