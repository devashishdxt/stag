use bounce::use_atom;
use yew::prelude::*;

use crate::atoms::StagAtom;

#[function_component(SignerList)]
pub fn signer_list() -> Html {
    let atom = use_atom::<StagAtom>();
    let signers = (*atom).signer.get_signers();

    match signers {
        Err(err) => html! {
            <h3>{ format!("Unable to fetch signer details: {:?}", err) }</h3>
        },
        Ok(signers) => {
            if signers.is_empty() {
                html! {
                    <h3>{ "No signers found!" }</h3>
                }
            } else {
                html! {
                    <table>
                        <tr>
                            <th>{ "Chain ID" }</th>
                            <th>{ "Account Address" }</th>
                        </tr>
                        { for signers.iter().map(|(chain_id, account_address)| {
                            html! {
                                <tr>
                                    <td>{ chain_id }</td>
                                    <td>{ account_address }</td>
                                </tr>
                            }
                        }) }
                    </table>
                }
            }
        }
    }
}
