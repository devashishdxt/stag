use web_sys::Event;
use yew::{classes, function_component, html, Classes, Properties, UseStateHandle};

const RADIO_CLASSES: &[&str] = &[
    "appearance-none",
    "h-4",
    "w-4",
    "mr-2",
    "rounded-lg",
    "border-2",
    "border-slate-700",
    "focus:bg-slate-700",
    "checked:bg-slate-700",
    "transition-all",
];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub name: String,
    pub placeholders: Vec<String>,
    pub value: UseStateHandle<String>,
    #[prop_or_default]
    pub class: Classes,
}

#[function_component(RadioInput)]
pub fn radio_input(props: &Props) -> Html {
    html! {
        <div class={props.class.clone()}>
            {
                for props.placeholders.clone().into_iter().map(|placeholder| {
                    html! {
                        <div class={classes!("flex", "items-center", "mb-2")}>
                            <input type="radio" name={props.name.clone()} class={classes!(RADIO_CLASSES)} onchange={
                                let value = props.value.clone();
                                let placeholder = placeholder.clone();

                                move |_: Event| {
                                    value.set(placeholder.clone());
                                }
                            } id={ placeholder.clone() } checked={*props.value == placeholder} />
                            <label for={ placeholder.clone() } class={classes!("text-slate-600")}>{ placeholder.clone() }</label>
                        </div>
                    }
                })
            }
        </div>
    }
}
