use anyhow::Result;
use async_trait::async_trait;

use super::{Event, EventHandler};

#[derive(Clone)]
pub struct TracingEventHandler;

#[cfg_attr(feature = "non-wasm", async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl EventHandler for TracingEventHandler {
    async fn handle_event(&self, event: Event) -> Result<()> {
        tracing::info!("{}", serde_json::to_string(&event)?);
        Ok(())
    }
}
