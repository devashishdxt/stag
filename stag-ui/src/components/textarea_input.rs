use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TextareaInputProps {
    pub label: String,
    pub placeholder: Option<String>,
    pub value: UseStateHandle<String>,
}

#[function_component(TextareaInput)]
pub fn textarea_input(props: &TextareaInputProps) -> Html {
    html! {
        <>
            <p>{ &props.label }</p>
            <textarea placeholder={ props.placeholder.clone().unwrap_or_else(|| props.label.clone()) } oninput={
                let value = props.value.clone();

                move |event: InputEvent| {
                    let target: HtmlInputElement = event.target_unchecked_into();
                    value.set(target.value());
                }
            } value={ (*props.value).clone() } />
        </>
    }
}
