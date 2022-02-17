use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::Serialize;
use url::Url;

use crate::trait_util::Base;

use super::JsonRpcClient;

#[derive(Default, Clone)]
pub struct ReqwestClient {
    client: Client,
    id: Arc<AtomicU32>,
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl JsonRpcClient for ReqwestClient {
    fn get_next_id(&self) -> u32 {
        self.id.fetch_add(1, Ordering::AcqRel)
    }

    async fn send_request<REQ>(&self, url: &Url, request: REQ) -> Result<serde_json::Value>
    where
        REQ: Serialize + Base,
    {
        let request = serde_json::to_string(&request)?;

        let response = self
            .client
            .post(url.clone())
            .header(CONTENT_TYPE, "application/json")
            .body(request)
            .send()
            .await
            .context("failed to send post request")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Unexpected response status code: {}",
                response.status()
            ));
        }

        let response = response.text().await?;

        Ok(serde_json::from_str(&response)?)
    }
}
