use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::componenets::{
    chain::{add_chain_form::AddChainForm, chain_list::ChainList},
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

#[function_component(Chains)]
pub fn chains(props: &Props) -> Html {
    html! {
        <Page name="Chains">
            <ChainList storage={props.storage.clone()} />
            <AddChainForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} rpc_client={props.rpc_client} event_handler={props.event_handler} />
        </Page>
    }
}
