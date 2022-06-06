use anyhow::Result;
use stag_api::{stag::Stag, storage::IndexedDb};
use tracing::error;
use yew::{classes, function_component, html, use_state, Html, Properties, UseStateHandle};

const NOTIFICATION_CLASSES: &[&str] = &["text-xl", "font-bold", "text-center", "pt-6", "pb-10"];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub storage: IndexedDb,
}

#[function_component(ChainList)]
pub fn chain_list(props: &Props) -> Html {
    let chains: UseStateHandle<Result<Vec<(String, bool)>>> = use_state(|| Ok(Vec::new()));

    {
        let chains = chains.clone();
        let storage = props.storage.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let all_chains = get_all_chains(storage).await;
            chains.set(all_chains);
        });
    }

    match &*chains {
        Ok(chains) => {
            if chains.is_empty() {
                html! {
                    <h3 class={classes!(NOTIFICATION_CLASSES)}>{ "No chains found!" }</h3>
                }
            } else {
                html! {
                    <table class={classes!("text-left", "mx-auto", "my-10")}>
                        <tr>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Chain ID" }</th>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Connection Status" }</th>
                        </tr>
                        { for chains.iter().map(|(chain_id, connected)| {
                            html! {
                                <tr>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ chain_id }</td>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ get_connection_status(*connected) }</td>
                                </tr>
                            }
                        }) }
                    </table>
                }
            }
        }
        Err(err) => {
            error!("Unable to fetch chains: {:?}", err);
            html! {
                <h3 class={classes!(NOTIFICATION_CLASSES, "text-red-500")}>{ "Unable to fetch chains!" }</h3>
            }
        }
    }
}

async fn get_all_chains(storage: IndexedDb) -> Result<Vec<(String, bool)>> {
    let stag = Stag::builder().with_storage(storage).await?.build();

    let chains = stag.get_all_chains(None, None).await?;

    Ok(chains
        .into_iter()
        .map(|chain| (chain.id.to_string(), chain.is_connected()))
        .collect())
}

fn get_connection_status(connected: bool) -> Html {
    if connected {
        html! {
            <span class={classes!("text-green-500")}><code>{ "CONNECTED" }</code></span>
        }
    } else {
        html! {
            <span class={classes!("text-red-500")}><code>{ "NOT CONNECTED" }</code></span>
        }
    }
}
