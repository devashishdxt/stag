use anyhow::Result;
use stag_api::{
    stag::Stag,
    storage::IndexedDb,
    types::{
        ics::core::ics24_host::identifier::{ChainId, PortId},
        operation::Operation,
    },
};
use tracing::error;
use wasm_bindgen::UnwrapThrowExt;
use web_sys::FocusEvent;
use yew::{classes, function_component, html, use_state, Callback, Properties, UseStateHandle};

use crate::componenets::{notification::NotificationData, text_input::TextInput};

const BUTTON_CLASSES: &[&str] = &[
    "px-8",
    "py-2",
    "rounded",
    "bg-slate-200",
    "hover:bg-slate-300",
    "hover:shadow",
    "transition-all",
];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub storage: IndexedDb,
}

#[function_component(GetHistoryForm)]
pub fn get_history_form(props: &Props) -> Html {
    let chain_id = use_state(|| "".to_string());
    let history = use_state(Vec::new);

    let on_submit = Callback::from({
        let notification = props.notification.clone();
        let storage = props.storage.clone();

        let chain_id = chain_id.clone();
        let history = history.clone();

        move |event: FocusEvent| {
            event.prevent_default();

            let notification = notification.clone();
            let storage = storage.clone();

            let chain_id = chain_id.clone();
            let history = history.clone();

            match chain_id.parse() {
                Ok(parsed_chain_id) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match get_history(storage, parsed_chain_id).await {
                            Ok(operations) => {
                                if operations.is_empty() {
                                    notification.set(Some(NotificationData::success(
                                        "No history found".to_string(),
                                    )));
                                }
                                chain_id.set("".to_string());
                                history.set(operations);
                            }
                            Err(err) => {
                                error!("Failed to fetch history: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to fetch history".to_string(),
                                )));
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("Invalid data: {:?}", err);
                    notification.set(Some(NotificationData::error(
                        "Invalid chain ID".to_string(),
                    )));
                }
            }
        }
    });

    html! {
        <>
            <div class={classes!("p-6")}>
                <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Get History" }</h2>
                <form class={classes!("pl-4")} onsubmit={on_submit}>
                    <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ chain_id.clone() } />
                    <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                    <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                        let history = history.clone();

                        move |_| {
                            chain_id.set("".to_string());
                            history.set(vec![]);
                        }
                    }>{ "Clear" }</button>
                </form>
            </div>
            {
                if !history.is_empty() {
                    html! {
                        <div class={classes!("border-t-2", "border-slate-400", "p-6")}>
                            <table class={classes!("text-left", "mx-auto", "my-10")}>
                                <tr>
                                    <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "ID" }</th>
                                    <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Channel" }</th>
                                    <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Transaction Hash" }</th>
                                    <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Operation" }</th>
                                    <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Time" }</th>
                                </tr>
                                { for history.iter().map(|operation| {
                                    html! {
                                        <tr>
                                            <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ operation.id }</td>
                                            <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ get_channel(&operation.port_id) }</td>
                                            <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ operation.transaction_hash.clone() }</td>
                                            <td class={classes!("px-4", "py-2", "border", "border-slate-600")}><code>{ serde_json::to_string_pretty(&operation.operation_type).expect_throw("Invalid operation type") }</code></td>
                                            <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ operation.created_at }</td>
                                        </tr>
                                    }
                                }) }
                            </table>
                        </div>
                    }
                } else {
                    html! {
                        <></>
                    }
                }
            }
        </>
    }
}

async fn get_history(storage: IndexedDb, chain_id: ChainId) -> Result<Vec<Operation>> {
    let stag = Stag::builder().with_storage(storage).await?.build();
    stag.get_history(&chain_id, None, None).await
}

fn get_channel(port_id: &PortId) -> String {
    let port_id = port_id.to_string();

    if port_id == "transfer" {
        "Transfer".to_string()
    } else if port_id.starts_with("icacontroller") {
        "ICA".to_string()
    } else {
        "Unknown".to_string()
    }
}
