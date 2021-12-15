use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use surf::{Client, Url};

#[derive(Debug)]
pub struct JsonRpcClient {
    client: Client,
    id: Arc<AtomicU32>,
}

impl JsonRpcClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub async fn send<REQ, RES>(&self, url: &Url, method: &str, params: REQ) -> Result<RES>
    where
        REQ: Serialize,
        RES: DeserializeOwned,
    {
        let id = self.id.fetch_add(1, Ordering::AcqRel);

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let value = self.send_request(url, request).await?;

        let response = value
            .as_object()
            .ok_or_else(|| anyhow!("invalid jsonrpc response"))?;

        if response.contains_key("error") {
            bail!("error jsonrpc response: {:?}", response["error"]);
        }

        let result = response["result"].clone();

        serde_json::from_value(result).context("jsonrpc response deserialization error")
    }

    async fn send_request<REQ>(&self, url: &Url, request: REQ) -> Result<serde_json::Value>
    where
        REQ: Serialize,
    {
        let request = serde_json::to_string(&request)?;

        let builder = self
            .client
            .post(url)
            .content_type(surf::http::mime::JSON)
            .body(request);

        let mut response = builder.send().await.map_err(|e| e.into_inner())?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Unexpected response status code: {}",
                response.status()
            ));
        }

        let response = response
            .body_string()
            .await
            .map_err(|err| err.into_inner())?;

        Ok(serde_json::from_str(&response)?)
    }
}
