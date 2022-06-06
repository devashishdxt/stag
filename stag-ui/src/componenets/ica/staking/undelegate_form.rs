use anyhow::Context;
use anyhow::Result;
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

#[derive(Clone)]
struct State {
    chain_id: UseStateHandle<String>,
    validator_address: UseStateHandle<String>,
    amount: UseStateHandle<String>,
    denom: UseStateHandle<String>,
    memo: UseStateHandle<String>,
}

impl State {
    fn parse(&self) -> Result<(ChainId, String, U256, Identifier, String), anyhow::Error> {
        let chain_id = self.chain_id.parse().context("Invalid chain ID")?;
        let validator_address = (*self.validator_address).clone();
        let amount = U256::from_dec_str(&self.amount).context("Invalid amount")?;
        let denom = (*self.denom).parse().context("Invalid denom")?;
        let memo = (*self.memo).clone();

        Ok((chain_id, validator_address, amount, denom, memo))
    }

    fn clear(&self) {
        self.chain_id.set("".to_string());
        self.validator_address.set("".to_string());
        self.amount.set("".to_string());
        self.denom.set("".to_string());
        self.memo.set("".to_string());
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            chain_id: use_state(|| "".to_string()),
            validator_address: use_state(|| "".to_string()),
            amount: use_state(|| "".to_string()),
            denom: use_state(|| "".to_string()),
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

#[function_component(UndelegateForm)]
pub fn undelegate_form(props: &Props) -> Html {
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
                "Undelegating tokens".to_string(),
            )));

            match state.parse() {
                Ok((chain_id, validator_address, amount, denom, memo)) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        match undelegate(
                            (*signer).clone(),
                            storage,
                            rpc_client,
                            event_handler,
                            chain_id,
                            validator_address,
                            amount,
                            denom,
                            memo,
                        )
                        .await
                        {
                            Ok(()) => {
                                state.clear();
                                notification.set(Some(NotificationData::success(
                                    "Successfully undelegated tokens".to_string(),
                                )));
                            }
                            Err(err) => {
                                error!("Failed to undeletage tokens: {:?}", err);
                                notification.set(Some(NotificationData::error(
                                    "Failed to undelegate tokens".to_string(),
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
        <div class={classes!("border-t-2", "border-slate-400", "p-6", "mt-6")}>
            <h2 class={classes!("text-2xl", "pb-6", "font-bold")}>{ "Undelegate Tokens" }</h2>
            <form class={classes!("pl-4")} onsubmit={on_submit}>
                <TextInput class={classes!("mb-4")} placeholder="Chain ID" value={ state.chain_id.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Validator address on host chain" value={ state.validator_address.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Amount of tokens to undelegate" value={ state.amount.clone() } />
                <TextInput class={classes!("mb-4")} placeholder="Denom of tokens to undelegate" value={ state.denom.clone() } />
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
async fn undelegate(
    signer: MnemonicSigner,
    storage: IndexedDb,
    rpc_client: ReqwestClient,
    event_handler: TracingEventHandler,
    chain_id: ChainId,
    validator_address: String,
    amount: U256,
    denom: Identifier,
    memo: String,
) -> Result<()> {
    let stag = Stag::builder()
        .with_signer(signer)?
        .with_storage(storage)
        .await?
        .with_rpc_client(rpc_client)
        .with_event_handler(event_handler)
        .build();

    stag.ica_undelegate(chain_id, None, validator_address, amount, denom, memo)
        .await
        .map(|_| ())
}
