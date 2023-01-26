#[cfg(feature = "solo-machine-v3")]
use crate::types::proto::ibc::lightclients::solomachine::v3::{
    SignBytes, TimestampedSignatureData,
};
use anyhow::Result;
use cosmos_sdk_proto::cosmos::tx::signing::v1beta1::{
    signature_descriptor::{
        data::{Single as SingleSignatureData, Sum as SignatureDataInner},
        Data as SignatureData,
    },
    SignMode,
};
#[cfg(not(feature = "solo-machine-v3"))]
use cosmos_sdk_proto::ibc::lightclients::solomachine::v2::{SignBytes, TimestampedSignatureData};

use crate::{
    signer::{Message, Signer},
    stag::StagContext,
    types::{
        chain_state::ChainState, ics::core::ics24_host::identifier::ChainId,
        proto_util::proto_encode,
    },
};

use super::common::to_u64_timestamp;

/// Signs a transaction with timestamp information
pub async fn timestamped_sign<C>(
    context: &C,
    chain_state: &ChainState,
    sign_bytes: SignBytes,
    request_id: Option<&str>,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let signature_data = sign(context, request_id, &chain_state.id, sign_bytes).await?;

    let timestamped_signature_data = TimestampedSignatureData {
        signature_data,
        timestamp: to_u64_timestamp(chain_state.consensus_timestamp)?,
    };

    proto_encode(&timestamped_signature_data)
}

/// Signs a transaction
pub async fn sign<C>(
    context: &C,
    request_id: Option<&str>,
    chain_id: &ChainId,
    sign_bytes: SignBytes,
) -> Result<Vec<u8>>
where
    C: StagContext,
    C::Signer: Signer,
{
    let sign_bytes = proto_encode(&sign_bytes)?;
    let signature = context
        .signer()
        .sign(request_id, chain_id, Message::SignBytes(&sign_bytes))
        .await?;

    let signature_data = SignatureData {
        sum: Some(SignatureDataInner::Single(SingleSignatureData {
            signature,
            mode: SignMode::Unspecified.into(),
        })),
    };

    proto_encode(&signature_data)
}
