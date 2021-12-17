use anyhow::Result;
use async_trait::async_trait;

use super::{Event, EventHandler};

pub struct TracingEventHandler;

#[async_trait]
impl EventHandler for TracingEventHandler {
    async fn handle(&self, event: Event) -> Result<()> {
        tracing::info!("{}", serde_json::to_string(&event)?);
        Ok(())
    }
}
