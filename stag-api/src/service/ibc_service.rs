use crate::event::{EventHandler, NoopEventHandler};

pub struct IbcService<E>
where
    E: EventHandler,
{
    event_handler: Option<E>,
}

impl IbcService<NoopEventHandler> {
    /// Creates a new instance of IBC service
    pub fn new() -> Self {
        Self {
            event_handler: None,
        }
    }
}

impl<E> IbcService<E>
where
    E: EventHandler,
{
    /// Adds an event handler to IBC service
    pub fn with_event_handler<NE>(self, event_handler: NE) -> IbcService<NE>
    where
        NE: EventHandler,
    {
        IbcService {
            event_handler: Some(event_handler),
        }
    }
}
