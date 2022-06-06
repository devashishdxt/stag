use anyhow::Result;
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use primitive_types::U256;
use serde::Serialize;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    tendermint::TendermintClient,
    transaction_builder::msg::packet::msg_receive_packet,
    types::{
        chain_state::ChainState,
        ics::core::ics24_host::identifier::{Identifier, PortId},
    },
};

#[derive(Debug, Serialize)]
struct TokenTransferPacketData {
    pub denom: String,
    // Ideally `amount` should be `U256` but `ibc-go` uses `protojson` which encodes `uint256` into `string`. So, using
    // `String` here to keep consistent wire format.
    pub amount: String,
    pub sender: String,
    pub receiver: String,
}

/// Creates and signs a `MsgRecvPacket` transaction.
pub async fn msg_mint<C>(
    context: &C,
    chain_state: &mut ChainState,
    amount: U256,
    denom: &Identifier,
    receiver: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let packet_data = TokenTransferPacketData {
        denom: denom.to_string(),
        amount: amount.to_string(),
        sender: context.signer().to_account_address(&chain_state.id).await?,
        receiver,
    };

    msg_receive_packet(
        context,
        chain_state,
        &PortId::transfer(),
        serde_json::to_vec(&packet_data)?,
        memo,
        request_id,
    )
    .await
}
