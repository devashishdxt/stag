use sealed::sealed;

use super::EventHandler;

#[cfg(feature = "tracing-event-handler")]
use super::tracing_event_handler::TracingEventHandler as TracingEventHandlerImpl;

#[sealed]
pub trait EventHandlerConfig {
    type EventHandler: EventHandler;

    fn into_event_handler(self) -> Self::EventHandler;
}

#[cfg(feature = "tracing-event-handler")]
/// Event handler backend using tracing
pub struct TracingEventHandler;

#[cfg(feature = "tracing-event-handler")]
#[sealed]
impl EventHandlerConfig for TracingEventHandler {
    type EventHandler = TracingEventHandlerImpl;

    fn into_event_handler(self) -> Self::EventHandler {
        TracingEventHandlerImpl {}
    }
}
