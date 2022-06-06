use web_sys::{Event, HtmlInputElement};
use yew::{classes, function_component, html, Classes, Properties, TargetCast, UseStateHandle};

const CHECKBOX_CLASSES: &[&str] = &[
    "appearance-none",
    "h-4",
    "w-4",
    "mr-2",
    "bg-slate-200",
    "rounded",
    "checked:rounded-lg",
    "checked:bg-slate-700",
    "transition-all",
];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub placeholder: String,
    pub value: UseStateHandle<bool>,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(CheckboxInput)]
pub fn checkbox_input(props: &Props) -> Html {
    html! {
        <div class={classes!("flex", "items-center", props.class.clone())}>
            <input type="checkbox" class={classes!(CHECKBOX_CLASSES)} onchange={
                let value = props.value.clone();

                move |event: Event| {
                    let target: HtmlInputElement = event.target_unchecked_into();
                    value.set(target.checked());
                }
            } checked={ *props.value } id={ props.placeholder.clone() } />
            <label for={ props.placeholder.clone() } class={classes!("text-slate-600")}>{ props.placeholder.clone() }</label>
        </div>
    }
}
