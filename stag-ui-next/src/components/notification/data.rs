use yew::{classes, html, Html};

#[derive(PartialEq)]
pub enum NotificationData {
    Success {
        message: &'static str,
    },
    Processing {
        message: &'static str,
    },
    Error {
        message: &'static str,
        details: Option<String>,
    },
}

impl Clone for NotificationData {
    fn clone(&self) -> Self {
        match self {
            Self::Success { message } => Self::Success { message },
            Self::Processing { message } => Self::Processing { message },
            Self::Error { message, details } => Self::Error {
                message,
                details: details.clone(),
            },
        }
    }
}

impl NotificationData {
    pub(super) fn is_dismissable(&self) -> bool {
        match self {
            Self::Success { .. } => true,
            Self::Processing { .. } => false,
            Self::Error { .. } => true,
        }
    }

    pub(super) fn message(&self) -> &'static str {
        match self {
            Self::Success { message } => message,
            Self::Processing { message } => message,
            Self::Error { message, .. } => message,
        }
    }

    pub(super) fn details(&self) -> Option<String> {
        match self {
            Self::Error { details, .. } => details.clone(),
            _ => None,
        }
    }

    pub(super) fn icon(&self) -> Html {
        match self {
            Self::Success { .. } => html! {
                <i class={classes!("fa-solid", "fa-check", "text-green-500")}></i>
            },
            Self::Processing { .. } => html! {
                <i class={classes!("fa-solid", "fa-bars-progress", "text-yellow-500")}></i>
            },
            Self::Error { .. } => html! {
                <i class={classes!("fa-solid", "fa-xmark", "text-red-500")}></i>
            },
        }
    }
}
