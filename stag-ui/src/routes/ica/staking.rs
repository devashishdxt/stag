use stag_api::{
    event::TracingEventHandler, signer::MnemonicSigner, storage::IndexedDb,
    tendermint::ReqwestClient,
};
use yew::{function_component, html, Properties, UseStateHandle};

use crate::{
    componenets::{
        ica::staking::{delegate_form::DelegateForm, undelegate_form::UndelegateForm},
        notification::NotificationData,
    },
    routes::page::Page,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
    pub event_handler: TracingEventHandler,
}

#[function_component(Staking)]
pub fn staking(props: &Props) -> Html {
    html! {
        <Page name="Staking">
            <DelegateForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} rpc_client={props.rpc_client} event_handler={props.event_handler} />
            <UndelegateForm notification={props.notification.clone()} signer={props.signer.clone()} storage={props.storage.clone()} rpc_client={props.rpc_client} event_handler={props.event_handler} />
        </Page>
    }
}
