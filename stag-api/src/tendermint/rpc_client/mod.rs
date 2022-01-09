//! JSON RPC client
mod builder;
#[cfg(feature = "reqwest-client")]
mod reqwest_client;
mod rpc_client_traits;

#[cfg(feature = "reqwest-client")]
pub use self::builder::ReqwestClient;
pub use self::{builder::JsonRpcConfig, rpc_client_traits::JsonRpcClient};

/// A no-op JSON RPC client
pub struct NoopRpcClient;
