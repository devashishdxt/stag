use anyhow::{anyhow, Context, Result};
use cosmos_sdk_proto::cosmos::{
    auth::v1beta1::QueryAccountRequest,
    base::v1beta1::Coin,
    tx::v1beta1::{
        mode_info::{Single, Sum},
        AuthInfo, Fee, ModeInfo, SignDoc, SignerInfo, TxBody, TxRaw,
    },
};
#[cfg(feature = "wasm")]
use grpc_web_client::Client;
#[cfg(feature = "non-wasm")]
use tonic::transport::Channel;
use url::Url;

use crate::{
    signer::{GetPublicKey, Message, Signer},
    stag::StagContext,
    types::{
        chain_state::ChainState,
        cosmos::account::Account,
        ics::core::ics24_host::identifier::ChainId,
        proto::cosmos::auth::v1beta1::query_client::QueryClient as AuthQueryClient,
        proto_util::{proto_encode, AnyConvert},
    },
};

pub async fn build<C, T>(
    context: &C,
    chain_state: &ChainState,
    messages: &[T],
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    T: AnyConvert,
{
    let tx_body = build_tx_body(messages, memo).context("unable to build transaction body")?;
    let tx_body_bytes = proto_encode(&tx_body)?;

    let (account_number, account_sequence) = get_account_details(context, chain_state).await?;

    let auth_info = build_auth_info(context, chain_state, account_sequence)
        .await
        .context("unable to build auth info")?;
    let auth_info_bytes = proto_encode(&auth_info)?;

    let signature = build_signature(
        context,
        tx_body_bytes.clone(),
        auth_info_bytes.clone(),
        &chain_state.id,
        account_number,
        request_id,
    )
    .await
    .context("unable to sign transaction")?;

    Ok(TxRaw {
        body_bytes: tx_body_bytes,
        auth_info_bytes,
        signatures: vec![signature],
    })
}

fn build_tx_body<T>(messages: &[T], memo: String) -> Result<TxBody>
where
    T: AnyConvert,
{
    let messages = messages
        .iter()
        .map(AnyConvert::to_any)
        .collect::<Result<_, _>>()?;

    Ok(TxBody {
        messages,
        memo,
        timeout_height: 0,
        extension_options: Default::default(),
        non_critical_extension_options: Default::default(),
    })
}

async fn get_account_details<C>(context: &C, chain_state: &ChainState) -> Result<(u64, u64)>
where
    C: StagContext,
    C::Signer: GetPublicKey,
{
    let mut query_client = get_auth_query_client(chain_state.config.grpc_addr.clone()).await?;

    let account_address = context.signer().to_account_address(&chain_state.id).await?;

    let response = query_client
        .account(QueryAccountRequest {
            address: account_address.clone(),
        })
        .await?
        .into_inner()
        .account
        .ok_or_else(|| anyhow!("unable to find account with address: {}", account_address))?;

    let account = Account::from_any(&response)?;
    let base_account = account
        .get_base_account()
        .ok_or_else(|| anyhow!("missing base account for address: {}", account_address))?;

    Ok((base_account.account_number, base_account.sequence))
}

async fn build_auth_info<C>(
    context: &C,
    chain_state: &ChainState,
    account_sequence: u64,
) -> Result<AuthInfo>
where
    C: StagContext,
    C::Signer: GetPublicKey,
{
    let signer_info = SignerInfo {
        public_key: Some(
            context
                .signer()
                .get_public_key(&chain_state.id)
                .await?
                .to_any()?,
        ),
        mode_info: Some(ModeInfo {
            sum: Some(Sum::Single(Single { mode: 1 })),
        }),
        sequence: account_sequence,
    };

    let fee = Fee {
        amount: vec![Coin {
            denom: chain_state.config.fee.denom.to_string(),
            amount: chain_state.config.fee.amount.to_string(),
        }],
        gas_limit: chain_state.config.fee.gas_limit,
        payer: "".to_owned(),
        granter: "".to_owned(),
    };

    Ok(AuthInfo {
        signer_infos: vec![signer_info],
        fee: Some(fee),
    })
}

async fn build_signature<C>(
    context: &C,
    body_bytes: Vec<u8>,
    auth_info_bytes: Vec<u8>,
    chain_id: &ChainId,
    account_number: u64,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let sign_doc = SignDoc {
        body_bytes,
        auth_info_bytes,
        chain_id: chain_id.to_string(),
        account_number,
    };

    let sign_doc_bytes = proto_encode(&sign_doc)?;

    context
        .signer()
        .sign(request_id, chain_id, Message::SignDoc(&sign_doc_bytes))
        .await
}

#[cfg(feature = "wasm")]
async fn get_auth_query_client(grpc_addr: Url) -> Result<AuthQueryClient<Client>> {
    let mut url = grpc_addr.to_string();

    if url.ends_with('/') {
        url.pop();
    }

    let grpc_client = Client::new(url);
    Ok(AuthQueryClient::new(grpc_client))
}

#[cfg(all(not(feature = "wasm"), feature = "non-wasm"))]
async fn get_auth_query_client(grpc_addr: Url) -> Result<AuthQueryClient<Channel>> {
    AuthQueryClient::connect(grpc_addr.to_string())
        .await
        .context("error when initializing grpc client")
}
