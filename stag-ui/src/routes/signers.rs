use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::componenets::{
    notification::NotificationData,
    signer::{
        add_signer_form::AddSignerForm, signer_list::SignerList,
        update_signer_form::UpdateSignerForm,
    },
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

#[function_component(Signers)]
pub fn signers(props: &Props) -> Html {
    html! {
        <Page name="Signers">
            <SignerList signer={props.signer.clone()} />
            <AddSignerForm notification={props.notification.clone()} signer={props.signer.clone()} />
            <UpdateSignerForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} rpc_client={props.rpc_client} event_handler={props.event_handler} />
        </Page>
    }
}
