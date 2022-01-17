use anyhow::{Context, Result};
use bounce::{use_atom, UseAtomHandle};
use stag_api::types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKeyAlgo};
use yew::prelude::*;
use yew_router::{history::History, hooks::use_history};

use crate::{
    atoms::{NotificationAtom, NotificationData, NotificationIcon, StagAtom},
    components::{TextInput, TextareaInput},
    routes::Route,
};

#[derive(Clone)]
struct ImportSignerState {
    chain_id: UseStateHandle<String>,
    mnemonic: UseStateHandle<String>,
    hd_path: UseStateHandle<String>,
    account_prefix: UseStateHandle<String>,
    algo: UseStateHandle<String>,
}

impl ImportSignerState {
    fn fill_defaults(&self) {
        self.chain_id.set("".to_string());
        self.mnemonic.set("".to_string());
        self.hd_path.set("m/44'/118'/0'/0/0".to_string());
        self.account_prefix.set("cosmos".to_string());
        self.algo.set("secp256k1".to_string());
    }

    fn add_signer(&self, atom: UseAtomHandle<StagAtom>) -> Result<()> {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain id")?;
        let algo: PublicKeyAlgo = self.algo.parse().context("Invalid public key algo")?;

        let mut inner = (*atom).clone();

        let signer = inner
            .signer
            .clone()
            .add_chain_config(
                chain_id,
                &self.mnemonic,
                Some(&self.hd_path),
                Some(&self.account_prefix),
                Some(algo),
            )
            .context("Unable to add signer")?;

        inner.signer = signer;

        atom.set(inner);

        Ok(())
    }
}

impl Default for ImportSignerState {
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

#[function_component(ImportSignerForm)]
pub fn add_signer_form() -> Html {
    let state = ImportSignerState::default();

    let atom = use_atom::<StagAtom>();
    let notification = use_atom::<NotificationAtom>();
    let history = use_history().unwrap();

    html! {
        <form onsubmit={ |event: FocusEvent| event.prevent_default() }>
            <button type="button" onclick = {
                let state = state.clone();
                move |_| state.fill_defaults()
            }>{ "Fill Defaults" }</button>
            <TextInput label="Chain ID" placeholder="Chain ID of IBC enabled chain" value={ state.chain_id.clone() }/>
            <TextareaInput label="Mnemonic" placeholder="Mnemonic passphrase" value={ state.mnemonic.clone() }/>
            <TextInput label="HD Path" placeholder="HD Path to derive bech32 addresses" value={ state.hd_path.clone() }/>
            <TextInput label="Address Prefix" placeholder="Bech32 address prefix" value={ state.account_prefix.clone() }/>
            <TextInput label="Public Key Algo" placeholder="Public key algorithm for chain (e.g. secp256k1)" value={ state.algo.clone() }/>
            <button type="submit" onclick = {
                move |_| {
                    let state = state.clone();
                    let atom = atom.clone();

                    notification.set(NotificationAtom {
                        data: Some(NotificationData {
                            message: "Importing signer".to_string(),
                            icon: NotificationIcon::Processing,
                            dismissable: false,
                        })
                    });

                    match state.add_signer(atom) {
                        Ok(()) => {
                            notification.set(NotificationAtom {
                                data: Some(NotificationData {
                                    message: "Imported new signer".to_string(),
                                    icon: NotificationIcon::Success,
                                    dismissable: true,
                                })
                            });
                            history.push(Route::Home);
                        },
                        Err(err) => {
                            notification.set(NotificationAtom {
                                data: Some(NotificationData {
                                    message: format!("Failed to add signer: {:?}", err),
                                    icon: NotificationIcon::Error,
                                    dismissable: true,
                                })
                            });
                        },
                    }
                }
            }>{ "Add Signer" }</button>
        </form>
    }
}
