use yew::{classes, html, Callback, Classes, Component, Context, Html, Properties};

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

pub struct RadioInput;

#[derive(PartialEq, Properties)]
pub struct RadioInputProps {
    pub name: &'static str,
    pub placeholders: &'static [&'static str],
    pub on_change: Callback<usize>,
    #[prop_or_default]
    pub class: Classes,
}

impl Component for RadioInput {
    type Message = usize;

    type Properties = RadioInputProps;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <div class={props.class.clone()}>
                {
                    for props.placeholders.iter().enumerate().map(|(i, placeholder)| {
                        html! {
                            <div class={classes!("flex", "items-center", "mb-2")}>
                                <input type="radio" name={props.name} class={classes!(RADIO_CLASSES)} id={ *placeholder } onchange={ctx.link().callback(move |_| i)} />
                                <label for={ *placeholder } class={classes!("text-slate-600")}>{ *placeholder }</label>
                            </div>
                        }
                    })
                }
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().on_change.emit(msg);
        false
    }
}
