use anyhow::{anyhow, Result};
use cosmos_sdk_proto::{
    cosmos::{base::v1beta1::Coin, tx::v1beta1::TxRaw},
    ibc::{applications::transfer::v1::MsgTransfer, core::client::v1::Height},
};
use primitive_types::U256;

use crate::{
    signer::{GetPublicKey, Signer},
    stag::StagContext,
    transaction_builder::tx::build,
    types::{
        chain_state::ChainState,
        ics::core::{
            ics02_client::height::IHeight,
            ics24_host::identifier::{Identifier, PortId},
        },
    },
};

pub async fn msg_transfer<C>(
    context: &C,
    chain_state: &ChainState,
    amount: U256,
    denom: &Identifier,
    receiver: String,
    memo: String,
    request_id: Option<&str>,
) -> Result<TxRaw>
where
    C: StagContext,
    C::Signer: Signer,
{
    let connection_details = chain_state.connection_details.as_ref().ok_or_else(|| {
        anyhow!(
            "connection details not found for chain with id {}",
            chain_state.id
        )
    })?;

    let port_id = PortId::transfer();

    let channel_details = connection_details
        .channels
        .get(&port_id)
        .ok_or_else(|| anyhow!("channel details for port {} not found", port_id))?;

    let denom = chain_state.get_ibc_denom(&port_id, denom)?;

    let sender = context.signer().to_account_address(&chain_state.id).await?;

    let message = MsgTransfer {
        source_port: channel_details.tendermint_port_id.to_string(),
        source_channel: channel_details.tendermint_channel_id.to_string(),
        token: Some(Coin {
            amount: amount.to_string(),
            denom,
        }),
        sender,
        receiver,
        timeout_height: Some(Height::new(0, u64::from(chain_state.sequence) + 1)),
        timeout_timestamp: 0,
    };

    build(context, chain_state, &[message], memo, request_id).await
}
