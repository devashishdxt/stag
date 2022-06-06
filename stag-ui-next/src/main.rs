use yew::{classes, props, Callback};

mod components;

use self::components::html::{Button, ButtonProps};

fn main() {
    yew::start_app_with_props::<Button>(props! {
        ButtonProps {
            text: "Submit",
            ty: "button",
            on_click: Some(Callback::from(|_| gloo_console::log!("clicked"))),
            class: classes!("mt-6"),
        }
    });
}
