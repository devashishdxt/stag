use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CheckboxInputProps {
    pub label: String,
    pub value: UseStateHandle<bool>,
}

#[function_component(CheckboxInput)]
pub fn checkbox_input(props: &CheckboxInputProps) -> Html {
    html! {
        <>
            <p>{ &props.label }</p>
            <input type="checkbox" oninput={
                let value = props.value.clone();

                move |event: InputEvent| {
                    let target: HtmlInputElement = event.target_unchecked_into();
                    value.set(target.checked());
                }
            } checked={ *props.value } />
        </>
    }
}
