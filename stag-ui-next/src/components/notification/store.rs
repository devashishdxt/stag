use yew_agent::{
    utils::store::{Store, StoreWrapper},
    AgentLink,
};

use super::NotificationData;

pub struct NotificationStore {
    pub data: Option<NotificationData>,
}

impl Store for NotificationStore {
    type Input = Option<NotificationData>;

    type Action = Option<NotificationData>;

    fn new() -> Self {
        Self { data: None }
    }

    fn handle_input(&self, link: AgentLink<StoreWrapper<Self>>, msg: Self::Input) {
        link.send_message(msg)
    }

    fn reduce(&mut self, msg: Self::Action) {
        self.data = msg;
    }
}
