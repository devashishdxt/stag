use anyhow::Result;
use stag_api::{stag::Stag, storage::IndexedDb, types::chain_state::ChainState};
use tracing::error;
use wasm_bindgen::UnwrapThrowExt;
use yew::{classes, function_component, html, use_state, Html, Properties, UseStateHandle};

const NOTIFICATION_CLASSES: &[&str] = &["text-xl", "font-bold", "text-center", "pt-6", "pb-10"];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub storage: IndexedDb,
}

#[function_component(ChannelList)]
pub fn channel_list(props: &Props) -> Html {
    let connected_chains: UseStateHandle<Result<Vec<ChainState>>> = use_state(|| Ok(Vec::new()));

    {
        let connected_chains = connected_chains.clone();
        let storage = props.storage.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let all_connected_chains = get_all_connected_chains(storage).await;
            connected_chains.set(all_connected_chains);
        });
    }

    match &*connected_chains {
        Ok(chains) => {
            if chains.is_empty() {
                html! {
                    <h3 class={classes!(NOTIFICATION_CLASSES)}>{ "No connected chains found!" }</h3>
                }
            } else {
                html! {
                    <table class={classes!("text-left", "mx-auto", "my-10")}>
                        <tr>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Chain ID" }</th>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Transfer Channel Status" }</th>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "ICA Channel Status" }</th>
                        </tr>
                        { for chains.iter().map(|chain| {
                            let (transfer_connected, ica_connected) = get_channel_statuses(chain);

                            html! {
                                <tr>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ chain.id.to_string() }</td>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ get_channel_status(transfer_connected) }</td>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ get_channel_status(ica_connected) }</td>
                                </tr>
                            }
                        }) }
                    </table>
                }
            }
        }
        Err(err) => {
            error!("Unable to fetch connected chains: {:?}", err);
            html! {
                <h3 class={classes!(NOTIFICATION_CLASSES, "text-red-500")}>{ "Unable to fetch connected chains!" }</h3>
            }
        }
    }
}

async fn get_all_connected_chains(storage: IndexedDb) -> Result<Vec<ChainState>> {
    let stag = Stag::builder().with_storage(storage).await?.build();
    let chains = stag.get_all_chains(None, None).await?;

    Ok(chains
        .into_iter()
        .filter(|chain| chain.is_connected())
        .collect())
}

/// Assumes that chain is already connected (panics otherwise)
fn get_channel_statuses(chain_state: &ChainState) -> (bool, bool) {
    let connection = chain_state
        .connection_details
        .as_ref()
        .expect_throw("chain must be connected");

    let channels = &connection.channels;

    let mut transfer_connected = false;
    let mut ica_connected = false;

    for (port_id, _) in channels.iter() {
        let port_id = port_id.to_string();

        if port_id == "transfer" {
            transfer_connected = true;
        } else if port_id.starts_with("icacontroller-") {
            ica_connected = true;
        }
    }

    (transfer_connected, ica_connected)
}

fn get_channel_status(created: bool) -> Html {
    if created {
        html! {
            <span class={classes!("text-green-500")}><code>{ "CREATED" }</code></span>
        }
    } else {
        html! {
            <span class={classes!("text-red-500")}><code>{ "NOT CREATED" }</code></span>
        }
    }
}
