use stag_api::signer::MnemonicSigner;
use tracing::error;
use yew::{classes, function_component, html, Properties, UseStateHandle};

const NOTIFICATION_CLASSES: &[&str] = &["text-xl", "font-bold", "text-center", "pt-6", "pb-10"];

#[derive(PartialEq, Properties)]
pub struct Props {
    pub signer: UseStateHandle<MnemonicSigner>,
}

#[function_component(SignerList)]
pub fn signer_list(props: &Props) -> Html {
    let current_signers = props.signer.get_signers();

    match current_signers {
        Ok(current_signers) => {
            if current_signers.is_empty() {
                html! {
                    <h3 class={classes!(NOTIFICATION_CLASSES)}>{ "No signers found!" }</h3>
                }
            } else {
                html! {
                    <table class={classes!("text-left", "mx-auto", "my-10")}>
                        <tr>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Chain ID" }</th>
                            <th class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ "Account Address" }</th>
                        </tr>
                        { for current_signers.iter().map(|(chain_id, account_address)| {
                            html! {
                                <tr>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}>{ chain_id }</td>
                                    <td class={classes!("px-4", "py-2", "border", "border-slate-600")}><code>{ account_address }</code></td>
                                </tr>
                            }
                        }) }
                    </table>
                }
            }
        }
        Err(err) => {
            error!("Unable to fetch signer details: {:?}", err);
            html! {
                <h3 class={classes!(NOTIFICATION_CLASSES, "text-red-500")}>{ "Unable to fetch signer details!" }</h3>
            }
        }
    }
}
