use yew::{classes, function_component, html, Html, Properties, UseStateHandle};

const BUTTON_CLASSES: &[&str] = &[
    "px-8",
    "py-2",
    "rounded",
    "bg-slate-200",
    "hover:bg-slate-300",
    "transition-all",
];

#[derive(Clone, Copy, PartialEq)]
pub enum NotificationType {
    Success,
    Processing,
    Error,
}

impl NotificationType {
    fn is_dismissable(&self) -> bool {
        match self {
            NotificationType::Success => true,
            NotificationType::Processing => false,
            NotificationType::Error => true,
        }
    }

    fn icon(&self) -> Html {
        match self {
            Self::Success => html! {
                <i class={classes!("fa-solid", "fa-check", "text-green-500")}></i>
            },
            Self::Processing => html! {
                <i class={classes!("fa-solid", "fa-bars-progress", "text-yellow-500")}></i>
            },
            Self::Error => html! {
                <i class={classes!("fa-solid", "fa-xmark", "text-red-500")}></i>
            },
        }
    }
}

#[derive(PartialEq)]
pub struct NotificationData {
    pub message: String,
    pub ty: NotificationType,
}

impl NotificationData {
    pub fn success(message: String) -> Self {
        Self {
            message,
            ty: NotificationType::Success,
        }
    }

    pub fn processing(message: String) -> Self {
        Self {
            message,
            ty: NotificationType::Processing,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            message,
            ty: NotificationType::Error,
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub data: UseStateHandle<Option<NotificationData>>,
}

#[function_component(Notification)]
pub fn notification(props: &Props) -> Html {
    match *props.data {
        None => html! {
            <></>
        },
        Some(ref data) => {
            html! {
                <div class={classes!("fixed", "left-0", "top-0", "bg-slate-600", "bg-opacity-70", "z-10", "h-screen", "w-screen", "text-center")}>
                <div class={classes!("inline-block", "px-20", "py-10", "bg-slate-100", "rounded", "mt-60", "text-center")}>
                    <div class={classes!("inline-block", "text-3xl", "pb-4")}>{ data.ty.icon() }</div>
                    <div class={classes!("text-lg", "pb-6")}>{ data.message.clone() }</div>
                    {
                        if data.ty.is_dismissable() {
                            html! {
                                <button class={classes!(BUTTON_CLASSES)} onclick={
                                    let data = props.data.clone();
                                    move |_| data.set(None)
                                }>{ "Dismiss" }</button>
                            }
                        } else {
                            html! { <></> }
                        }
                    }

                </div>
            </div>
            }
        }
    }
}
