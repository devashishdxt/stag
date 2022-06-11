mod reader;
mod writer;

pub use self::{reader::StoreReader, writer::StoreWriter};

use std::{collections::HashSet, rc::Rc};

use yew_agent::{Agent, AgentLink, Context, Dispatched, Dispatcher, HandlerId};

pub struct Store<T>
where
    T: Clone + Default + 'static,
{
    /// Data contained in store
    data: Rc<T>,
    /// Currently subscribed components and agents
    handlers: HashSet<HandlerId>,
    /// Link to itself so Store::handle_input can send actions to reducer
    link: AgentLink<Self>,
    /// A circular dispatcher to itself so the store is not removed
    _dispatcher: Dispatcher<Self>,
}

impl<T> Agent for Store<T>
where
    T: Clone + Default + 'static,
{
    type Reach = Context<Self>;

    type Message = T;

    type Input = T;

    type Output = Rc<T>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            data: Default::default(),
            handlers: Default::default(),
            link,
            _dispatcher: Self::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        self.data = Rc::new(msg);

        for handler in self.handlers.iter() {
            self.link.respond(*handler, self.data.clone());
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        self.link.send_message(msg);
    }

    fn connected(&mut self, id: HandlerId) {
        self.handlers.insert(id);
        self.link.respond(id, self.data.clone());
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.handlers.remove(&id);
    }
}
