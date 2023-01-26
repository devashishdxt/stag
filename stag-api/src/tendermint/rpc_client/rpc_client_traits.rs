use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use url::Url;

use crate::trait_util::Base;

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
/// Trait that must be implemented by all the JSON RPC backends
pub trait JsonRpcClient: Base {
    /// Returns the next ID for json rpc request
    fn get_next_id(&self) -> u32;

    /// Sends a json rpc request to the given url
    async fn send_request<REQ>(&self, url: &Url, request: REQ) -> Result<serde_json::Value>
    where
        REQ: Serialize + Base;

    /// Create and send a json rpc request to the given url
    async fn send<REQ, RES>(&self, url: &Url, method: &str, params: REQ) -> Result<RES>
    where
        REQ: Serialize + Send + Sync,
        RES: DeserializeOwned + Send + Sync,
    {
        let id = self.get_next_id();

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let value = self.send_request(url, request).await?;

        let response = value.as_object().context("invalid jsonrpc response")?;

        if response.contains_key("error") {
            bail!("error jsonrpc response: {:?}", response["error"]);
        }

        let result = response["result"].clone();

        serde_json::from_value(result).context(format!(
            "jsonrpc response deserialization error [method: {}]",
            method
        ))
    }
}

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl<T> JsonRpcClient for &T
where
    T: JsonRpcClient,
{
    fn get_next_id(&self) -> u32 {
        (**self).get_next_id()
    }

    async fn send_request<REQ>(&self, url: &Url, request: REQ) -> Result<serde_json::Value>
    where
        REQ: Serialize + Base,
    {
        (**self).send_request(url, request).await
    }
}
