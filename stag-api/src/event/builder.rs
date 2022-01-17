use sealed::sealed;

use crate::trait_util::Base;

use super::EventHandler;

#[cfg(feature = "tracing-event-handler")]
use super::tracing_event_handler::TracingEventHandler as TracingEventHandlerImpl;

#[sealed]
/// Configuration for event handler
pub trait EventHandlerConfig: Base {
    /// Concrete event handler type that this config will produce
    type EventHandler: EventHandler;

    /// Create concrete event handler from this config
    fn into_event_handler(self) -> Self::EventHandler;
}

#[cfg_attr(feature = "doc", doc(cfg(feature = "tracing-event-handler")))]
#[cfg(feature = "tracing-event-handler")]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
/// Event handler backend using tracing
pub struct TracingEventHandler;

#[cfg_attr(feature = "doc", doc(cfg(feature = "tracing-event-handler")))]
#[cfg(feature = "tracing-event-handler")]
#[sealed]
impl EventHandlerConfig for TracingEventHandler {
    type EventHandler = TracingEventHandlerImpl;

    fn into_event_handler(self) -> Self::EventHandler {
        TracingEventHandlerImpl {}
    }
}
