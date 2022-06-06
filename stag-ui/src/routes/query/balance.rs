use stag_api::{signer::MnemonicSigner, storage::IndexedDb};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::{
    componenets::{chain::get_balance_form::GetBalanceForm, notification::NotificationData},
    routes::page::Page,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
}

#[function_component(Balance)]
pub fn balance(props: &Props) -> Html {
    html! {
        <Page name="Balance">
            <GetBalanceForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} />
        </Page>
    }
}
