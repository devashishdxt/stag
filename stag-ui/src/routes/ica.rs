use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::componenets::{
    ica::{get_ica_address_form::GetIcaAddressForm, send_form::SendForm},
    notification::NotificationData,
};

use super::page::Page;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
    pub event_handler: TracingEventHandler,
}

#[function_component(Ica)]
pub fn ica(props: &Props) -> Html {
    html! {
        <Page name="Interchain Accounts">
            <SendForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} rpc_client={props.rpc_client} event_handler={props.event_handler} />
            <GetIcaAddressForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} />
        </Page>
    }
}
