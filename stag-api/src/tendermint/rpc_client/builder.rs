use async_trait::async_trait;
use sealed::sealed;

use crate::trait_util::Base;

#[cfg(feature = "reqwest-client")]
use super::reqwest_client::ReqwestClient as ReqwestClientImpl;
use super::JsonRpcClient;

#[cfg_attr(feature = "doc", doc(cfg(feature = "reqwest-client")))]
#[cfg(feature = "reqwest-client")]
/// Json RPC backend using `reqwest`
pub struct ReqwestClient;

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
#[sealed]
/// Configuration for json rpc client
pub trait JsonRpcConfig: Base {
    /// Concrete json rpc client type that this config will produce
    type Client: JsonRpcClient;

    /// Create concrete json rpc client from this config
    fn into_client(self) -> Self::Client;
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "reqwest-client")))]
#[cfg(feature = "reqwest-client")]
#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
#[sealed]
impl JsonRpcConfig for ReqwestClient {
    type Client = ReqwestClientImpl;

    fn into_client(self) -> Self::Client {
        ReqwestClientImpl::default()
    }
}
