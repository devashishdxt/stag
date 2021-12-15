use anyhow::Result;
use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
use tendermint_light_client::types::{LightBlock, ValidatorSet};
use tendermint_rpc::endpoint::{broadcast, commit, status, validators};
use url::Url;

use crate::{tendermint::json_rpc_client::JsonRpcClient, types::proto::proto_encode};

/// Responsible for broadcasting transactions to tendermint using JSON-RPC
#[derive(Debug)]
pub struct TendermintClient {
    rpc_client: JsonRpcClient,
    url: Url,
}

impl TendermintClient {
    /// Creates a new tendermint RPC client
    pub fn new(url: Url) -> Self {
        Self {
            rpc_client: JsonRpcClient::new(),
            url,
        }
    }

    pub fn set_url(&mut self, url: Url) {
        self.url = url;
    }

    pub async fn light_block(&self, height: Option<u32>) -> Result<LightBlock> {
        let signed_header = self.commit(height).await?.signed_header;

        let height = signed_header.header.height;
        let proposer_address = signed_header.header.proposer_address;

        let validators = ValidatorSet::with_proposer(
            self.all_validators(Some(height.value().try_into()?))
                .await?
                .validators,
            proposer_address,
        )?;

        let next_validators = ValidatorSet::without_proposer(
            self.all_validators(Some(height.increment().value().try_into()?))
                .await?
                .validators,
        );

        let provider = self.status().await?.node_info.id;

        Ok(LightBlock {
            signed_header,
            validators,
            next_validators,
            provider,
        })
    }

    pub async fn all_validators(&self, height: Option<u32>) -> Result<validators::Response> {
        let mut validators = Vec::new();

        let mut page_number = 1;

        loop {
            let response = self.validators(height, Some(page_number), Some(30)).await?;
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

    pub async fn validators(
        &self,
        height: Option<u32>,
        page_number: Option<usize>,
        per_page: Option<u8>,
    ) -> Result<validators::Response> {
        let mut request = validators::Request::default();
        request.height = height.map(Into::into);
        request.page = page_number.map(Into::into);
        request.per_page = per_page.map(Into::into);

        self.rpc_client.send(&self.url, "validators", request).await
    }

    pub async fn commit(&self, height: Option<u32>) -> Result<commit::Response> {
        let height = height.map(Into::into);

        self.rpc_client
            .send(&self.url, "commit", commit::Request { height })
            .await
    }

    pub async fn status(&self) -> Result<status::Response> {
        self.rpc_client
            .send(&self.url, "status", status::Request)
            .await
    }

    /// Broadcasts transaction and returns transaction hash and deliver_tx response
    pub async fn broadcast_tx(&self, transaction: TxRaw) -> Result<broadcast::tx_commit::Response> {
        self.rpc_client
            .send(
                &self.url,
                "broadcast_tx_commit",
                broadcast::tx_commit::Request {
                    tx: proto_encode(&transaction)?.into(),
                },
            )
            .await
    }
}
