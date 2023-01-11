use anyhow::Result;
use async_trait::async_trait;
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use tendermint_light_client::types::{LightBlock, ValidatorSet};
use tendermint_rpc::endpoint::{broadcast, commit, status, validators};
use url::Url;

use crate::{tendermint::rpc_client::JsonRpcClient, types::proto_util::proto_encode};

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
/// Helper trait to send a transaction to a tendermint node (auto-implemented for json rpc client)
pub trait TendermintClient: JsonRpcClient {
    /// Sends status request to tendermint node
    async fn status(&self, url: &Url) -> Result<status::Response> {
        self.send(url, "status", status::Request).await
    }

    /// Sends broadcast_tx request to tendermint node
    async fn broadcast_tx(
        &self,
        url: &Url,
        transaction: TxRaw,
    ) -> Result<broadcast::tx_commit::Response> {
        self.send(
            url,
            "broadcast_tx_commit",
            broadcast::tx_commit::Request {
                tx: proto_encode(&transaction)?,
            },
        )
        .await
    }

    /// Sends commit request to tendermint node
    async fn commit(&self, url: &Url, height: Option<u32>) -> Result<commit::Response> {
        let height = height.map(Into::into);
        self.send(url, "commit", commit::Request { height }).await
    }

    /// Sends validators request to tendermint node
    async fn validators(
        &self,
        url: &Url,
        height: Option<u32>,
        page_number: Option<usize>,
        per_page: Option<u8>,
    ) -> Result<validators::Response> {
        let mut request = validators::Request::default();
        request.height = height.map(Into::into);
        request.page = page_number.map(Into::into);
        request.per_page = per_page.map(Into::into);

        self.send(url, "validators", request).await
    }

    /// Fetches all the validators from tendermint node
    async fn all_validators(&self, url: &Url, height: Option<u32>) -> Result<validators::Response> {
        let mut validators = Vec::new();

        let mut page_number = 1;

        loop {
            let response = self
                .validators(url, height, Some(page_number), Some(30))
                .await?;
            validators.extend(response.validators);

            if validators.len() as i32 == response.total {
                return Ok(validators::Response::new(
                    response.block_height,
                    validators,
                    response.total,
                ));
            }

            page_number += 1;
        }
    }

    /// Fetches light block from tendermint node
    async fn light_block(&self, url: &Url, height: Option<u32>) -> Result<LightBlock> {
        let signed_header = self.commit(url, height).await?.signed_header;

        let height = signed_header.header.height;
        let proposer_address = signed_header.header.proposer_address;

        let validators = ValidatorSet::with_proposer(
            self.all_validators(url, Some(height.value().try_into()?))
                .await?
                .validators,
            proposer_address,
        )?;

        let next_validators = ValidatorSet::without_proposer(
            self.all_validators(url, Some(height.increment().value().try_into()?))
                .await?
                .validators,
        );

        let provider = self.status(url).await?.node_info.id;

        Ok(LightBlock {
            signed_header,
            validators,
            next_validators,
            provider,
        })
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl<C> TendermintClient for C where C: JsonRpcClient {}
