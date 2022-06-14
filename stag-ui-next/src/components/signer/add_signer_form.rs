use std::rc::Rc;

use anyhow::{Context as _, Result};
use stag_api::{
    signer::MnemonicSigner,
    types::{ics::core::ics24_host::identifier::ChainId, public_key::PublicKeyAlgo},
};
use web_sys::FocusEvent;
use yew::{classes, html, Component, Context, Html};

use crate::{
    components::{
        html::{Button, TextInput},
        notification::NotificationData,
    },
    store::{StoreReader, StoreWriter},
};

pub enum AddSignerFormMsg {
    NewSigner(Rc<MnemonicSigner>),
    NewChainId(String),
    NewMnemonic(String),
    NewHdPath(String),
    NewAccountPrefix(String),
    NewAlgo(String),
    AddSigner,
    Clear,
}

pub struct AddSignerForm {
    signer: Rc<MnemonicSigner>,
    notification_writer: StoreWriter<Option<NotificationData>>,
    _signer_reader: StoreReader<MnemonicSigner>,
    signer_writer: StoreWriter<MnemonicSigner>,

    chain_id: String,
    mnemonic: String,
    hd_path: String,
    account_prefix: String,
    algo: String,
}

impl AddSignerForm {
    fn clear(&mut self) {
        self.chain_id.clear();
        self.mnemonic.clear();
        self.hd_path.clear();
        self.account_prefix.clear();
        self.algo.clear();
    }

    fn add_signer(&mut self) -> bool {
        self.notification_writer
            .set(Some(NotificationData::Processing {
                message: "Adding signer",
            }));

        match self.parse() {
            Ok((chain_id, mnemonic, hd_path, account_prefix, algo)) => {
                let mut new_signer = self.signer.as_ref().clone();

                match new_signer.add_chain_config(
                    chain_id,
                    &mnemonic,
                    hd_path.as_deref(),
                    account_prefix.as_deref(),
                    algo,
                ) {
                    Ok(_) => {
                        self.signer_writer.set(new_signer);
                        self.clear();
                        self.notification_writer
                            .set(Some(NotificationData::Success {
                                message: "Signer added",
                            }));

                        true
                    }
                    Err(err) => {
                        self.notification_writer.set(Some(NotificationData::Error {
                            message: "Failed to add signer",
                            details: Some(format!("{:?}", err)),
                        }));
                        false
                    }
                }
            }
            Err(err) => {
                self.notification_writer.set(Some(NotificationData::Error {
                    message: "Failed to add signer",
                    details: Some(format!("{:?}", err)),
                }));
                false
            }
        }
    }

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
        let mnemonic = self.mnemonic.clone();
        let hd_path = if self.hd_path.is_empty() {
            None
        } else {
            Some(self.hd_path.clone())
        };
        let account_prefix = if self.account_prefix.is_empty() {
            None
        } else {
            Some(self.account_prefix.clone())
        };
        let algo: Option<PublicKeyAlgo> = if self.algo.is_empty() {
            None
        } else {
            Some(self.algo.parse().context("Invalid public key algo")?)
        };

        Ok((chain_id, mnemonic, hd_path, account_prefix, algo))
    }
}

impl Component for AddSignerForm {
    type Message = AddSignerFormMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            signer: Default::default(),
            notification_writer: StoreWriter::new(),
            _signer_reader: StoreReader::new(ctx.link().callback(AddSignerFormMsg::NewSigner)),
            signer_writer: StoreWriter::new(),

            chain_id: Default::default(),
            mnemonic: Default::default(),
            hd_path: Default::default(),
            account_prefix: Default::default(),
            algo: Default::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("border-t-2", "border-slate-400", "p-6")}>
                <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Add Signer" }</h2>
                <form class={classes!("pl-4")} onsubmit={ctx.link().callback(|event: FocusEvent| {
                    event.prevent_default();
                    AddSignerFormMsg::AddSigner
                })}>
                    <TextInput class={classes!("mb-4")} placeholder="Chain ID" on_change={ctx.link().callback(AddSignerFormMsg::NewChainId)} />
                    <TextInput class={classes!("mb-4")} placeholder="Mnemonic phrase" on_change={ctx.link().callback(AddSignerFormMsg::NewMnemonic)} />
                    <TextInput class={classes!("mb-4")} placeholder="HD Path to derive bech32 addresses (default: 'm/44'/118'/0'/0/0')" on_change={ctx.link().callback(AddSignerFormMsg::NewHdPath)} />
                    <TextInput class={classes!("mb-4")} placeholder="Bech32 address prefix (default: 'cosmos')" on_change={ctx.link().callback(AddSignerFormMsg::NewAccountPrefix)} />
                    <TextInput class={classes!("mb-4")} placeholder="Public key algorithm for chain (e.g. 'secp256k1' | 'eth-secp256k1') (default: 'secp256k1')" on_change={ctx.link().callback(AddSignerFormMsg::NewAlgo)} />
                    <Button ty="submit" text="Submit" />
                    <Button ty="button" text="Clear" class={classes!("ml-6")} on_click={ctx.link().callback(|()| AddSignerFormMsg::Clear)} />
                </form>
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddSignerFormMsg::NewSigner(signer) => {
                self.signer = signer;
                false
            }
            AddSignerFormMsg::NewChainId(chain_id) => {
                self.chain_id = chain_id;
                false
            }
            AddSignerFormMsg::NewMnemonic(mnemonic) => {
                self.mnemonic = mnemonic;
                false
            }
            AddSignerFormMsg::NewHdPath(hd_path) => {
                self.hd_path = hd_path;
                false
            }
            AddSignerFormMsg::NewAccountPrefix(account_prefix) => {
                self.account_prefix = account_prefix;
                false
            }
            AddSignerFormMsg::NewAlgo(algo) => {
                self.algo = algo;
                false
            }
            AddSignerFormMsg::AddSigner => self.add_signer(),
            AddSignerFormMsg::Clear => {
                self.clear();
                true
            }
        }
    }
}
