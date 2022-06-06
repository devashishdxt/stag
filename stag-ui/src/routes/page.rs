use yew::{classes, function_component, html, Children, Properties};

const HEADER_CLASSES: &[&str] = &[
    "bg-slate-100",
    "text-center",
    "py-8",
    "text-3xl",
    "font-bold",
    "shadow-lg",
];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub name: String,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Page)]
pub fn signers(props: &Props) -> Html {
    html! {
        <>
            <div class={classes!(HEADER_CLASSES)}>{ &props.name }</div>
            <div class={classes!("p-6")}>{ props.children.clone() }</div>
        </>
    }
}
