use stag_api::{signer::MnemonicSigner, storage::IndexedDb};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::{
    componenets::{ica::get_ica_address_form::GetIcaAddressForm, notification::NotificationData},
    routes::page::Page,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
}

#[function_component(Ica)]
pub fn ica(props: &Props) -> Html {
    html! {
        <Page name="Interchain Accounts">
            <GetIcaAddressForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} />
        </Page>
    }
}
