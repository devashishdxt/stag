#[cfg(feature = "wasm")]
mod indexed_db_storage;

use anyhow::Result;
use async_trait::async_trait;
use tendermint::node::Id as NodeId;

use crate::{ChainConfig, ChainId, ChainState};

#[cfg_attr(not(feature = "wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
pub trait Storage {
    async fn add_chain_state(
        &self,
        chain_id: ChainId,
        node_id: NodeId,
        chain_config: ChainConfig,
    ) -> Result<()>;

    async fn get_chain_state(&self, chain_id: &ChainId) -> Result<Option<ChainState>>;
}
