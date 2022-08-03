use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_channel::{Receiver, Sender};
use thiserror::Error;

use crate::component::{Component, ComponentId};

pub trait BusMessage: 'static + Send + Sync + Clone + fmt::Debug {}

#[derive(Debug, Error)]
pub enum MessageBusError {

}

struct MessageBusState<M: BusMessage> {
    receivers: HashMap<ComponentId, Vec<Receiver<M>>>,
    senders: HashMap<ComponentId, Vec<Sender<M>>>,
}

pub struct MessageBus<M: BusMessage> {
    id: ComponentId,
    state: Arc<Mutex<MessageBusState<M>>>
}

impl<M: BusMessage> MessageBus<M> {
    pub fn new(name: &str) -> Self {
        let state = MessageBusState { 
            receivers: HashMap::new(),
            senders: HashMap::new(),
        };
        Self {
            id: ComponentId::new(name),
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn connect(&self, sender: impl Component, receiver: impl Component) -> Result<()> {
        let sender_id = sender.id();
        let receiver_id = receiver.id();
        let (tx_sender_to_receiver, rx_receiver_from_sender) = async_channel::unbounded();

        {
            let mut state = self.state.lock().expect("receivers lock was poisoned");
            let receiver_entry = state.receivers.entry(receiver_id).or_default();
            receiver_entry.push(rx_receiver_from_sender);
            let sender_entry = state.senders.entry(sender_id).or_default();
            sender_entry.push(tx_sender_to_receiver);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fmt;
    use super::{BusMessage, MessageBus};

    #[derive(Clone)]
    struct TestMessage {
        contents: String,
    }

    impl BusMessage for TestMessage {}

    impl fmt::Debug for TestMessage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("TestMessage").field("contents", &self.contents).finish()
        }
    }

    #[test]
    fn new_works() {
        let _ = MessageBus::<TestMessage>::new("test bus");
    }
}
