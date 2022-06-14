use std::rc::Rc;

use stag_api::signer::MnemonicSigner;
use tracing::error;
use yew::{classes, html, Component, Context, Html};

use crate::store::StoreReader;

pub struct SignerList {
    signer: Rc<MnemonicSigner>,
    _signer_reader: StoreReader<MnemonicSigner>,
}

impl Component for SignerList {
    type Message = Rc<MnemonicSigner>;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            signer: Default::default(),
            _signer_reader: StoreReader::new(ctx.link().callback(|signer| signer)),
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let current_signers = self.signer.get_signers();

        match current_signers {
            Ok(current_signers) => {
                if current_signers.is_empty() {
                    html! {
                        <h3 class={classes!("text-xl", "font-bold", "text-center", "pt-6", "pb-10")}>{ "No signers found!" }</h3>
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
                    <h3 class={classes!("text-xl", "font-bold", "text-center", "pt-6", "pb-10", "text-red-500")}>{ "Unable to fetch signer details!" }</h3>
                }
            }
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        self.signer = msg;
        true
    }
}
