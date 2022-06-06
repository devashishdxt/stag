use yew::{classes, html, Callback, Classes, Component, Context, Html, Properties};

const BUTTON_CLASSES: &[&str] = &[
    "px-8",
    "py-2",
    "rounded",
    "bg-slate-200",
    "hover:bg-slate-300",
    "hover:shadow",
    "active:bg-slate-400",
    "transition-all",
];

pub struct Button;

#[derive(PartialEq, Properties)]
pub struct ButtonProps {
    pub text: &'static str,
    pub ty: &'static str,
    pub on_click: Option<Callback<()>>,
    #[prop_or_default]
    pub class: Classes,
}

impl Component for Button {
    type Message = ();

    type Properties = ButtonProps;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <button type={props.ty} class={classes!(BUTTON_CLASSES, props.class.clone())} onclick={ctx.link().callback(|_| ())}>{ props.text }</button>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, _: Self::Message) -> bool {
        if let Some(ref on_click) = ctx.props().on_click {
            on_click.emit(());
        }
        false
    }
}
