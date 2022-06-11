use yew::Callback;
use yew_agent::{Agent, Bridge, Bridged};

use super::Store;

pub struct StoreReader<T>
where
    T: Clone + Default + 'static,
{
    _bridge: Box<dyn Bridge<Store<T>>>,
}

impl<T> StoreReader<T>
where
    T: Clone + Default + 'static,
{
    /// Creates a new store reader
    pub fn new(callback: Callback<<Store<T> as Agent>::Output>) -> Self {
        Self {
            _bridge: Store::bridge(callback),
        }
    }
}
