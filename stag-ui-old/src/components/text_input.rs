use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TextInputProps {
    pub label: String,
    pub placeholder: Option<String>,
    pub value: UseStateHandle<String>,
}

#[function_component(TextInput)]
pub fn text_input(props: &TextInputProps) -> Html {
    html! {
        <>
            <p>{ &props.label }</p>
            <input type="text" placeholder={ props.placeholder.clone().unwrap_or_else(|| props.label.clone()) } oninput={
                let value = props.value.clone();

                move |event: InputEvent| {
                    let target: HtmlInputElement = event.target_unchecked_into();
                    value.set(target.value());
                }
            } value={ (*props.value).clone() } />
        </>
    }
}
