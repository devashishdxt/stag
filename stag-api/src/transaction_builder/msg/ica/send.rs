use anyhow::{anyhow, bail, Result};
use cosmos_sdk_proto::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin, tx::v1beta1::TxRaw};
use primitive_types::U256;

use crate::{
    signer::Signer,
    stag::StagContext,
    storage::Storage,
    tendermint::TendermintClient,
    types::{
        chain_state::ChainState,
        ics::core::ics24_host::identifier::{Identifier, PortId},
        proto_util::AnyConvert,
    },
};

use super::submit::msg_submit;

/// Creates and signs a `MsgRecvPacket` transaction.
#[allow(clippy::too_many_arguments)]
pub async fn msg_send<C>(
    context: &C,
    chain_state: &mut ChainState,
    solo_machine_port_id: &PortId,
    to_address: String,
    amount: U256,
    denom: &Identifier,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
    C::Storage: Storage,
    C::RpcClient: TendermintClient,
{
    let solo_machine_connection_id = match chain_state.connection_details {
        Some(ref details) => details.solo_machine_connection_id.clone(),
        None => bail!("No connection details found"),
    };

    let ica_address = context
        .storage()
        .get_ica_address(&solo_machine_connection_id, solo_machine_port_id)
        .await?
        .ok_or_else(|| {
            anyhow!(
                "No ICA address found for connection {} and port {}",
                solo_machine_connection_id,
                solo_machine_port_id
            )
        })?;

    let msg = MsgSend {
        from_address: ica_address,
        to_address,
        amount: vec![Coin {
            amount: amount.to_string(),
            denom: denom.to_string(),
        }],
    }
    .to_any()?;

    msg_submit(
        context,
        chain_state,
        solo_machine_port_id,
        vec![msg],
        memo,
        request_id,
    )
    .await
}
