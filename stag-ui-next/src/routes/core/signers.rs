use yew::{html, Component, Context, Html};

use crate::{
    components::signer::{add_signer_form::AddSignerForm, signer_list::SignerList},
    routes::page::Page,
};

pub struct Signers;

impl Component for Signers {
    type Message = ();

    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <Page name="Signers">
                <SignerList />
                <AddSignerForm />
            </Page>
        }
    }
}
