use stag_api::signer::MnemonicSigner;
use yew::{function_component, html, Properties, UseStateHandle};

use crate::componenets::{
    notification::NotificationData,
    signer::{add_signer_form::AddSignerForm, signer_list::SignerList},
};

use super::page::Page;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
}

#[function_component(Signers)]
pub fn signers(props: &Props) -> Html {
    html! {
        <Page name="Signers">
            <SignerList signer={props.signer.clone()} />
            <AddSignerForm notification={props.notification.clone()} signer={props.signer.clone()} />
        </Page>
    }
}
