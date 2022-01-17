use yew::prelude::*;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <nav>
            <h1>{ "NOT FOUND" }</h1>
        </nav>
    }
}
