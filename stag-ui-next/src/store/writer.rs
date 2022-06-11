use yew_agent::{Dispatched, Dispatcher};

use super::Store;

pub struct StoreWriter<T>
where
    T: Clone + Default + 'static,
{
    dispatcher: Dispatcher<Store<T>>,
}

impl<T> StoreWriter<T>
where
    T: Clone + Default + 'static,
{
    /// Creates a new store writer
    pub fn new() -> Self {
        Self {
            dispatcher: Store::dispatcher(),
        }
    }

    /// Sets the store data
    pub fn set(&mut self, data: T) {
        self.dispatcher.send(data)
    }
}
