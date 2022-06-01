use anyhow::{Context, Result};
use primitive_types::U256;
use stag_api::{
    event::TracingEventHandler,
    signer::MnemonicSigner,
    stag::Stag,
    storage::IndexedDb,
    tendermint::ReqwestClient,
    types::ics::core::ics24_host::identifier::{ChainId, Identifier},
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

#[derive(Clone, PartialEq)]
struct State {
    chain_id: UseStateHandle<String>,
    amount: UseStateHandle<String>,
    denom: UseStateHandle<String>,
    receiver: UseStateHandle<String>,
    memo: UseStateHandle<String>,
}

impl State {
    fn parse(&self) -> Result<(ChainId, U256, Identifier, Option<String>, String)> {
        let chain_id: ChainId = self.chain_id.parse().context("Invalid chain ID")?;
        let amount: U256 = U256::from_dec_str(&self.amount).context("Invalid amount")?;
        let denom: Identifier = self.denom.parse().context("Invalid denom")?;
        let receiver: Option<String> = if self.receiver.is_empty() {
            None
        } else {
            Some((*self.receiver).clone())
        };
        let memo = (*self.memo).clone();

        Ok((chain_id, amount, denom, receiver, memo))
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
        self.amount.set("".to_string());
        self.denom.set("".to_string());
        self.receiver.set("".to_string());
        self.memo.set("".to_string());
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            amount: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
            receiver: use_state(|| "".to_string()),
            memo: use_state(|| "".to_string()),
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub notification: UseStateHandle<Option<NotificationData>>,
    pub signer: UseStateHandle<MnemonicSigner>,
    pub storage: IndexedDb,
    pub rpc_client: ReqwestClient,
    pub event_handler: TracingEventHandler,
}

#[function_component(MintForm)]
pub fn mint_form(props: &Props) -> Html {
    let state = State::default();

    let on_submit = Callback::from({
        let notification = props.notification.clone();
        let signer = props.signer.clone();
        let storage = props.storage.clone();
        let rpc_client = props.rpc_client;
        let event_handler = props.event_handler;

        let state = state.clone();

        move |event: FocusEvent| {
            event.prevent_default();

            let notification = notification.clone();
            let signer = signer.clone();
            let storage = storage.clone();

            let state = state.clone();

            notification.set(Some(NotificationData::processing(
                "Minting tokens".to_string(),
            )));

            match state.parse() {
                Ok((chain_id, amount, denom, receiver, memo)) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match mint(
                            (*signer).clone(),
                            storage,
                            rpc_client,
                            event_handler,
                            chain_id,
                            amount,
                            denom,
                            receiver,
                            memo,
                        )
                        .await
                        {
                            Ok(()) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(
                                    "Successfully minted tokens".to_string(),
                                )));
                            }
                            Err(err) => {
                                error!("Failed to mint tokens: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to mint tokens".to_string(),
                                )));
                            }
                        }
                    });
                }
                Err(err) => {
                    error!("Invalid data: {:?}", err);
                    notification.set(Some(NotificationData::error(err.to_string())));
                }
            }
        }
    });

    html! {
        <div class={classes!("p-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Mint Tokens" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Amount of tokens to mint" value={ state.amount.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Denom of tokens to mint" value={ state.denom.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Receiver address on IBC enabled chain (if this is not provided, tokens will be sent to signer's address)" value={ state.receiver.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Memo to send in IBC transaction (optional)" value={ state.memo.clone() } />
                <button type="submit" class={classes!(BUTTON_CLASSES)}>{ "Submit" }</button>
                <button type="button" class={classes!(BUTTON_CLASSES, "ml-6")} onclick={
                    move |_| state.clear()
                }>{ "Clear" }</button>
            </form>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
async fn mint(
    signer: MnemonicSigner,
    storage: IndexedDb,
    rpc_client: ReqwestClient,
    event_handler: TracingEventHandler,
    chain_id: ChainId,
    amount: U256,
    denom: Identifier,
    receiver: Option<String>,
    memo: String,
) -> Result<()> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(rpc_client)
        .with_event_handler(event_handler)
        .build();

    stag.mint(chain_id, None, amount, denom, receiver, memo)
        .await
        .map(|_| ())
}
