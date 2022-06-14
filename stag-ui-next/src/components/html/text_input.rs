use web_sys::HtmlInputElement;
use yew::{
    classes, html, Callback, Classes, Component, Context, Html, InputEvent, Properties, TargetCast,
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

pub struct TextInput;

#[derive(PartialEq, Properties)]
pub struct TextInputProps {
    pub id: Option<String>,
    pub placeholder: &'static str,
    pub on_change: Callback<String>,
    #[prop_or_default]
    pub class: Classes,
}

impl Component for TextInput {
    type Message = String;

    type Properties = TextInputProps;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(move |event: InputEvent| {
            let target: HtmlInputElement = event.target_unchecked_into();
            target.value()
        });

        let placeholder = ctx.props().placeholder;
        let class = classes!(INPUT_CLASSES, ctx.props().class.clone());

        html! {
            <input id={ctx.props().id.clone().unwrap_or_default()} type="text" {class} {oninput} {placeholder} />
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().on_change.emit(msg);
        false
    }
}
