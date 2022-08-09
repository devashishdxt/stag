use anyhow::Result;
use async_trait::async_trait;

use super::{Event, EventHandler};

#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[derive(Clone)]
pub struct TracingEventHandler;

#[cfg_attr(all(not(feature = "wasm"), feature = "non-wasm"), async_trait)]
#[cfg_attr(feature = "wasm", async_trait(?Send))]
impl EventHandler for TracingEventHandler {
    async fn handle_event(&self, event: Event) -> Result<()> {
        tracing::info!("{}", serde_json::to_string(&event)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_event_handler() {
        let event_handler = TracingEventHandler;
        assert!(event_handler.handle_event(Event::Test).await.is_ok());
    }
}
