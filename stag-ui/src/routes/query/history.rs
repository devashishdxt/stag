use stag_api::storage::IndexedDb;
use yew::{function_component, html, Properties, UseStateHandle};

use crate::{
    componenets::{chain::get_history_form::GetHistoryForm, notification::NotificationData},
    routes::page::Page,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub storage: IndexedDb,
}

#[function_component(History)]
pub fn history(props: &Props) -> Html {
    html! {
        <Page name="History">
            <GetHistoryForm notification={props.notification.clone()} storage={props.storage.clone()} />
        </Page>
    }
}
