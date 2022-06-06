use anyhow::Result;
use cosmos_sdk_proto::{
    cosmos::tx::v1beta1::TxRaw,
    ibc::applications::interchain_accounts::v1::{CosmosTx, Type},
};
use prost_types::Any;
use serde::Serialize;

use crate::{
    signer::Signer,
    stag::StagContext,
    tendermint::TendermintClient,
    transaction_builder::msg_receive_packet,
    types::{
        chain_state::ChainState, ics::core::ics24_host::identifier::PortId,
        proto_util::proto_encode,
    },
};

#[derive(Debug, Serialize)]
pub struct InterchainAccountPacketData {
    #[serde(rename = "type")]
    pub ty: i32,
    pub data: Vec<u8>,
    pub memo: String,
}

pub async fn msg_submit<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    messages: Vec<Any>,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let cosmos_tx = CosmosTx { messages };
    let data = proto_encode(&cosmos_tx)?;

    let packet_data = InterchainAccountPacketData {
        ty: Type::ExecuteTx.into(),
        data,
        memo: memo.clone(),
    };

    msg_receive_packet(
        context,
        chain_state,
        solo_machine_port_id,
        serde_json::to_vec(&packet_data)?,
        memo,
        request_id,
    )
    .await
}
