use anyhow::{Context, Result};
use stag_api::{
    signer::MnemonicSigner,
    types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKeyAlgo},
};
use tracing::error;
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
    pub signer: UseStateHandle<MnemonicSigner>,
}

#[derive(Clone)]
struct State {
    chain_id: UseStateHandle<String>,
    mnemonic: UseStateHandle<String>,
    hd_path: UseStateHandle<String>,
    account_prefix: UseStateHandle<String>,
    algo: UseStateHandle<String>,
}

impl State {
    #[allow(clippy::type_complexity)]
    fn parse(
        &self,
    ) -> Result<(
        ChainId,
        String,
        Option<String>,
        Option<String>,
        Option<PublicKeyAlgo>,
    )> {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain id")?;
        let mnemonic = (*self.mnemonic).clone();
        let hd_path = if self.hd_path.is_empty() {
            None
        } else {
            Some((*self.hd_path).clone())
        };
        let account_prefix = if self.account_prefix.is_empty() {
            None
        } else {
            Some((*self.account_prefix).clone())
        };
        let algo: Option<PublicKeyAlgo> = if self.algo.is_empty() {
            None
        } else {
            Some(self.algo.parse().context("Invalid public key algo")?)
        };

        Ok((chain_id, mnemonic, hd_path, account_prefix, algo))
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
        self.mnemonic.set("".to_string());
        self.hd_path.set("".to_string());
        self.account_prefix.set("".to_string());
        self.algo.set("".to_string());
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            mnemonic: use_state(|| "".to_string()),
            hd_path: use_state(|| "".to_string()),
            account_prefix: use_state(|| "".to_string()),
            algo: use_state(|| "".to_string()),
        }
    }
}

#[function_component(AddSignerForm)]
pub fn add_signer_form(props: &Props) -> Html {
    let state = State::default();

    let on_submit = Callback::from({
        let notification = props.notification.clone();
        let signer = props.signer.clone();
        let state = state.clone();

        move |event: FocusEvent| {
            event.prevent_default();

            notification.set(Some(NotificationData::processing(
                "Adding signer".to_string(),
            )));

            match state.parse() {
                Ok((chain_id, mnemonic, hd_path, account_prefix, algo)) => {
                    let mut new_signer = (*signer).clone();

                    match new_signer.add_chain_config(
                        chain_id,
                        &mnemonic,
                        hd_path.as_deref(),
                        account_prefix.as_deref(),
                        algo,
                    ) {
                        Ok(_) => {
                            signer.set(new_signer);
                            state.clear();
                            notification
                                .set(Some(NotificationData::success("Signer added".to_string())))
                        }
                        Err(err) => {
                            error!("Failed to add signer: {:?}", err);
                            notification.set(Some(NotificationData::error(
                                "Failed to add signer".to_string(),
                            )));
                        }
                    }
                }
                Err(err) => {
                    error!("Invalid data: {:?}", err);
                    notification.set(Some(NotificationData::error("Invalid data".to_string())));
                }
            }
        }
    });

    html! {
        <div class={classes!("border-t-2", "border-slate-400", "p-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Add Signer" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Mnemonic phrase" value={ state.mnemonic.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="HD Path to derive bech32 addresses (default: 'm/44'/118'/0'/0/0')" value={ state.hd_path.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Bech32 address prefix (default: 'cosmos')" value={ state.account_prefix.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Public key algorithm for chain (e.g. 'secp256k1' | 'eth-secp256k1') (default: 'secp256k1')" value={ state.algo.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}
