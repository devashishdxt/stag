use web_sys::HtmlInputElement;
use yew::{
    classes, function_component, html, Classes, InputEvent, Properties, TargetCast, UseStateHandle,
};

const INPUT_CLASSES: &[&str] = &[
    "border",
    "border-slate-400",
    "rounded",
    "py-2",
    "px-4",
    "outline-none",
    "w-full",
];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub placeholder: String,
    pub value: UseStateHandle<String>,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(TextInput)]
pub fn text_input(props: &Props) -> Html {
    html! {
        <input type="text" class={classes!(INPUT_CLASSES, props.class.clone())} placeholder={ props.placeholder.clone() } oninput={
            let value = props.value.clone();

            move |event: InputEvent| {
                let target: HtmlInputElement = event.target_unchecked_into();
                value.set(target.value());
            }
        } value={ (*props.value).clone() }/>
    }
}
