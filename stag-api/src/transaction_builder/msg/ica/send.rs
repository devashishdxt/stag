use anyhow::Result;
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;

use crate::{
    signer::Signer,
    stag::StagContext,
    tendermint::TendermintClient,
    types::{chain_state::ChainState, ics::core::ics24_host::identifier::PortId},
};

use super::submit::msg_submit;

/// Creates and signs a `MsgRecvPacket` transaction.
pub async fn msg_send<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::RpcClient: TendermintClient,
{
    let data = vec![]; // TODO: prepare message data

    msg_submit(
        context,
        chain_state,
        solo_machine_port_id,
        data,
        memo,
        request_id,
    )
    .await
}
