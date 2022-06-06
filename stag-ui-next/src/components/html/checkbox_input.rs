use web_sys::{Event, HtmlInputElement};
use yew::{classes, html, Callback, Classes, Component, Context, Html, Properties, TargetCast};

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

pub struct CheckboxInput;

#[derive(PartialEq, Properties)]
pub struct CheckboxInputProps {
    pub placeholder: &'static str,
    pub on_change: Callback<bool>,
    #[prop_or_default]
    pub class: Classes,
}

impl Component for CheckboxInput {
    type Message = bool;

    type Properties = CheckboxInputProps;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().callback(move |event: Event| {
            let target: HtmlInputElement = event.target_unchecked_into();
            target.checked()
        });

        let props = ctx.props();

        html! {
            <div class={classes!("flex", "items-center", props.class.clone())}>
                <input type="checkbox" class={classes!(CHECKBOX_CLASSES)} {onchange} />
                <label for={ props.placeholder } class={classes!("text-slate-600")}>{ props.placeholder }</label>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().on_change.emit(msg);
        false
    }
}
