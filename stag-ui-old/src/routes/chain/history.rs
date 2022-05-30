use stag_api::types::ics::core::ics24_host::identifier::ChainId;
use yew::prelude::*;

use crate::{
    components::{GetHistoryForm, HistoryList, Navbar, Notification},
    routes::Route,
};

#[derive(Clone, PartialEq, Properties)]
pub struct HistoryProps {
    pub chain_id: Option<ChainId>,
}

#[function_component(History)]
pub fn history(props: &HistoryProps) -> Html {
    html! {
        <>
            <Navbar active = { Route::Chain } />
            {
                match props.chain_id {
                    None => html! {
                        <GetHistoryForm />
                    },
                    Some(ref chain_id) => html! {
                        <section>
                            <HistoryList chain_id = { chain_id.clone() } />
                        </section>
                    },
                }
            }
            <Notification />
        </>
    }
}
