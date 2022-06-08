use yew_agent::{utils::store::StoreWrapper, Dispatched, Dispatcher};

use super::{NotificationData, NotificationStore};

pub struct NotificationSender {
    dispatcher: Dispatcher<StoreWrapper<NotificationStore>>,
}

impl Default for NotificationSender {
    fn default() -> Self {
        Self {
            dispatcher: StoreWrapper::dispatcher(),
        }
    }
}

impl NotificationSender {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn send(&mut self, data: NotificationData) {
        self.dispatcher.send(Some(data))
    }
}
