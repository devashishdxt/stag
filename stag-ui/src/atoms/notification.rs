use bounce::Atom;

#[derive(Default, PartialEq, Atom, Clone)]
pub struct NotificationAtom {
    pub data: Option<NotificationData>,
}

#[derive(PartialEq, Clone)]
pub struct NotificationData {
    pub message: String,
    pub icon: NotificationIcon,
    pub dismissable: bool,
}

#[derive(PartialEq, Clone, Copy)]
pub enum NotificationIcon {
    Success,
    Processing,
    Error,
}
