use anyhow::Result;
use bounce::use_atom;
use stag_api::{
    event::TracingEventHandler,
    stag::Stag,
    storage::IndexedDb,
    types::{
        ics::core::ics24_host::identifier::ChainId,
        operation::{Operation, OperationType},
    },
};
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon, StagAtom},
    routes::ChainRoute,
};

#[derive(Clone, PartialEq, Properties)]
pub struct HistoryListProps {
    pub chain_id: ChainId,
}

#[function_component(HistoryList)]
pub fn history_list(props: &HistoryListProps) -> Html {
    let transaction_history = use_state(Vec::<Operation>::new);

    let props = props.clone();
    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    let transaction_history_clone = transaction_history.clone();

    wasm_bindgen_futures::spawn_local(async move {
        match get_transaction_history((*atom).storage.clone(), atom.event_handler, &props.chain_id)
            .await
        {
            Ok(operations) => {
                transaction_history_clone.set(operations);
            }
            Err(err) => {
                notification.set(NotificationAtom {
                    data: Some(NotificationData {
                        message: format!("Failed to fetch transaction history: {:?}", err),
                        icon: NotificationIcon::Success,
                        dismissable: true,
                    }),
                });

                history.push(ChainRoute::History);
            }
        }
    });

    if transaction_history.is_empty() {
        html! {
            <h3>{ "No transaction history found!" }</h3>
        }
    } else {
        html! {
            <table class="small-fonts">
                <tr>
                    <th>{ "Address" }</th>
                    <th>{ "Amount" }</th>
                    <th>{ "Type" }</th>
                    <th>{ "Transaction Hash" }</th>
                    <th>{ "Time" }</th>
                </tr>
                {
                    transaction_history.iter().map(|operation| {
                        html! {
                            <tr>
                                <td>{ operation.address.clone() }</td>
                                <td>{ format!("{} {}", operation.amount, operation.denom) }</td>
                                <td>{
                                    match operation.operation_type {
                                        OperationType::Mint => html! {
                                            <span class="green">{ "MINT" }</span>
                                        },
                                        OperationType::Burn => html! {
                                            <span class="red">{ "BURN" }</span>
                                        },
                                    }
                                }</td>
                                <td>{ operation.transaction_hash.clone() }</td>
                                <td>{ operation.created_at }</td>
                            </tr>
                        }
                    }).collect::<Html>()
                }
            </table>
        }
    }
}

async fn get_transaction_history(
    storage: IndexedDb,
    event_handler: TracingEventHandler,
    chain_id: &ChainId,
) -> Result<Vec<Operation>> {
    let stag = Stag::builder()
        .with_storage(storage)
        .await?
        .with_event_handler(event_handler)
        .build();
    stag.get_history(chain_id, None, None).await
}
