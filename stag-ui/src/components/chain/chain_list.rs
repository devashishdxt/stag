use anyhow::Result;
use bounce::use_atom;
use stag_api::{
    signer::SignerConfig, stag::Stag, storage::IndexedDb, tendermint::ReqwestClient,
    types::ics::core::ics24_host::identifier::ChainId,
};
use yew::prelude::*;

use crate::atoms::StagAtom;

#[function_component(ChainList)]
pub fn chain_list() -> Html {
    let chains: UseStateHandle<Result<Vec<(ChainId, bool)>>> = use_state(|| Ok(Vec::new()));

    let atom = use_atom::<StagAtom>();

    let chains_clone = chains.clone();

    wasm_bindgen_futures::spawn_local(async move {
        let new_chains = get_chains(
            (*atom).signer.clone(),
            (*atom).storage.clone(),
            atom.rpc_client,
        )
        .await;

        chains_clone.set(new_chains);
    });

    match *chains {
        Err(ref err) => html! {
            <h3>{ format!("Unable to fetch chain details: {:?}", err) }</h3>
        },
        Ok(ref chains) => {
            if chains.is_empty() {
                html! {
                    <h3>{ "No chains found!" }</h3>
                }
            } else {
                html! {
                    <table>
                        <tr>
                            <th>{ "Chain ID" }</th>
                            <th>{ "Connection Status" }</th>
                        </tr>
                        {
                            chains.iter().map(|(chain_id, is_connected)| {
                                html! {
                                    <tr>
                                        <td>{ chain_id }</td>
                                        <td>{
                                            if *is_connected {
                                                html! {
                                                    <span class="green">{ "CONNECTED" }</span>
                                                }
                                            } else {
                                                html! {
                                                    <span class="red">{ "NOT CONNECTED" }</span>
                                                }
                                            }
                                        }</td>
                                    </tr>
                                }
                            }).collect::<Html>()
                        }
                    </table>
                }
            }
        }
    }
}

async fn get_chains<S>(
    signer: S,
    storage: IndexedDb,
    rpc_client: ReqwestClient,
) -> Result<Vec<(ChainId, bool)>>
where
    S: SignerConfig,
{
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(rpc_client)
        .build();

    let chains = stag.get_all_chains(None, None).await?;

    Ok(chains
        .into_iter()
        .map(|chain| {
            let is_connected = chain.is_connected();
            (chain.id, is_connected)
        })
        .collect())
}
