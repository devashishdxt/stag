use yew::{classes, html, Children, Component, Context, Html, Properties};

const HEADER_CLASSES: &[&str] = &[
    "bg-slate-100",
    "text-center",
    "py-8",
    "text-3xl",
    "font-bold",
    "shadow-lg",
];

#[derive(PartialEq, Properties)]
pub struct PageProps {
    pub name: &'static str,
    #[prop_or_default]
    pub children: Children,
}

pub struct Page;

impl Component for Page {
    type Message = ();

    type Properties = PageProps;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        html! {
            <div class={classes!("flex")}>
                // <Sidebar />
                <div class={classes!("w-full")}>
                    <div class={classes!(HEADER_CLASSES)}>{ props.name }</div>
                    <div class={classes!("p-6")}>{ props.children.clone() }</div>
                </div>
            </div>
        }
    }
}
