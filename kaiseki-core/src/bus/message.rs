use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_channel::{Receiver, Sender};
use thiserror::Error;

use crate::component::{Component, ComponentId};

pub trait BusMessage: 'static + Send + Sync + Clone + fmt::Debug {}

#[derive(Debug, Error, PartialEq)]
pub enum MessageBusError {
    #[error("sender {0} is disconnected from receiver {1}")]
    Disconnected(ComponentId, ComponentId),
    #[error("no receivers connected for sender {0}")]
    NoReceiversForSender(ComponentId),
    #[error("no senders connected for receiver {0}")]
    NoSendersToReceiver(ComponentId),
}

struct MessageBusState<M: BusMessage> {
    receivers: HashMap<ComponentId, Vec<(ComponentId, Receiver<M>)>>,
    senders: HashMap<ComponentId, Vec<(ComponentId, Sender<M>)>>,
}

#[derive(Clone)]
pub struct MessageBus<M: BusMessage> {
    id: ComponentId,
    state: Arc<Mutex<MessageBusState<M>>>,
}

impl<M: BusMessage> Component for MessageBus<M> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
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

    pub fn connect(&self, sender: &impl Component, receiver: &impl Component) -> Result<()> {
        let sender_id = sender.id();
        let receiver_id = receiver.id();
        let (tx_sender_to_receiver, rx_receiver_from_sender) = async_channel::unbounded();

        {
            let mut state = self
                .state
                .lock()
                .expect("MessageBus state lock was poisoned");
            let receiver_entry = state.receivers.entry(receiver_id.clone()).or_default();
            receiver_entry.push((sender_id.clone(), rx_receiver_from_sender));
            let sender_entry = state.senders.entry(sender_id.clone()).or_default();
            sender_entry.push((receiver_id.clone(), tx_sender_to_receiver));
        }
        Ok(())
    }

    pub async fn send(&self, sender_id: &ComponentId, message: M) -> Result<(), MessageBusError> {
        let state = self
            .state
            .lock()
            .expect("MessageBus state lock was poisoned in send()");
        let senders = state
            .senders
            .get(sender_id)
            .ok_or(MessageBusError::NoReceiversForSender(sender_id.clone()))?;
        for (receiver_id, tx) in senders {
            match tx.send(message.clone()).await {
                Ok(_) => tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, message),
                Err(_) => {
                    return Err(MessageBusError::Disconnected(
                        sender_id.clone(),
                        receiver_id.clone(),
                    ))
                }
            }
        }
        Ok(())
    }

    pub async fn recv(&self, receiver_id: &ComponentId) -> Result<M, MessageBusError> {
        let state = self
            .state
            .lock()
            .expect("MessageBus state lock was poisoned in recv()");
        let receivers = state
            .receivers
            .get(receiver_id)
            .ok_or(MessageBusError::NoSendersToReceiver(receiver_id.clone()))?;

        // TODO: use a FuturesUnordered or similar here to wait on all at once, return first
        let message = receivers[0].1.recv().await.unwrap();
        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::{BusMessage, Component, ComponentId, MessageBus, MessageBusError};
    use std::fmt;

    #[derive(Clone, PartialEq)]
    struct TestMessage {
        contents: String,
    }

    impl BusMessage for TestMessage {}

    impl fmt::Debug for TestMessage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("TestMessage")
                .field("contents", &self.contents)
                .finish()
        }
    }

    struct TestComponent {
        id: ComponentId,
        bus: MessageBus<TestMessage>,
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }
    }

    impl TestComponent {
        pub fn new(name: &str, bus: &MessageBus<TestMessage>) -> Self {
            Self {
                id: ComponentId::new(name),
                bus: bus.clone(),
            }
        }
    }

    fn setup() -> ([TestComponent; 5], MessageBus<TestMessage>) {
        let bus = MessageBus::<TestMessage>::new("test bus");
        let components = [
            TestComponent::new("a", &bus),
            TestComponent::new("b", &bus),
            TestComponent::new("c", &bus),
            TestComponent::new("d", &bus),
            TestComponent::new("e", &bus),
        ];
        (components, bus)
    }

    #[test]
    fn new_works() {
        let _ = MessageBus::<TestMessage>::new("test bus");
    }

    #[test]
    fn connect_works() {
        let ([a, b, _, _, _], bus) = setup();
        bus.connect(&a, &b).expect("couldn't connect from a to b");

        let state = bus.state.lock().unwrap();
        let sender_value = state
            .senders
            .get(a.id())
            .expect("state.senders should have an entry for a");
        assert_eq!(
            sender_value.len(),
            1,
            "state.senders[a.id()] should have length 1"
        );

        let receiver_value = state
            .receivers
            .get(b.id())
            .expect("state.receivers should have an entry for b");
        assert_eq!(
            receiver_value.len(),
            1,
            "state.receivers[b.id()] should have length 1"
        );
    }

    #[tokio::test]
    async fn send_recv_works() {
        let ([a, b, c, d, e], bus) = setup();
        let a_msg = TestMessage {
            contents: String::from("message from a"),
        };
        assert_eq!(
            a.bus.send(a.id(), a_msg.clone()).await,
            Err(MessageBusError::NoReceiversForSender(a.id().clone()))
        );

        bus.connect(&a, &b).expect("couldn't connect from a to b");
        bus.connect(&a, &c).expect("couldn't connect from a to c");
        bus.connect(&d, &e).expect("couldn't connect from d to e");

        a.bus.send(a.id(), a_msg.clone()).await.unwrap();
        let b_msg = b.bus.recv(b.id()).await.unwrap();
        assert_eq!(a_msg, b_msg);
        let c_msg = c.bus.recv(c.id()).await.unwrap();
        assert_eq!(a_msg, c_msg);
    }
}
