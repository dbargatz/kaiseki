use futures::{stream::FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use async_channel::{Receiver, Sender, TryRecvError};
use thiserror::Error;

use crate::component::{Component, ComponentId};

pub trait BusMessage: 'static + Send + Sync + Clone + fmt::Debug {}

#[derive(Debug, Error, PartialEq)]
pub enum MessageBusError {
    #[error("sender {0} is disconnected from receiver {1}")]
    Disconnected(ComponentId, ComponentId),
    #[error("no messages available for receiver {0}")]
    NoMessagesAvailable(ComponentId),
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
    state: Arc<RwLock<MessageBusState<M>>>,
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
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn connect(&self, sender: &impl Component, receiver: &impl Component) -> Result<()> {
        let sender_id = sender.id();
        let receiver_id = receiver.id();
        let (tx_sender_to_receiver, rx_receiver_from_sender) = async_channel::unbounded();

        {
            let mut state = self
                .state
                .write()
                .expect("MessageBus state lock was poisoned");
            let receiver_entry = state.receivers.entry(receiver_id.clone()).or_default();
            receiver_entry.push((sender_id.clone(), rx_receiver_from_sender));
            let sender_entry = state.senders.entry(sender_id.clone()).or_default();
            sender_entry.push((receiver_id.clone(), tx_sender_to_receiver));
        }
        Ok(())
    }

    pub async fn send(&self, sender: &impl Component, message: M) -> Result<(), MessageBusError> {
        let state = self
            .state
            .read()
            .expect("MessageBus state lock was poisoned in send()");
        let sender_id = sender.id();
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

    pub async fn recv(&self, receiver: &impl Component) -> Result<M, MessageBusError> {
        let state = self
            .state
            .read()
            .expect("MessageBus state lock was poisoned in recv()");
        let receiver_id = receiver.id();
        let receivers = state
            .receivers
            .get(receiver_id)
            .ok_or(MessageBusError::NoSendersToReceiver(receiver_id.clone()))?;

        let mut futures = FuturesUnordered::new();
        for (sender_id, rx) in receivers {
            futures.push(async {
                let result = rx.recv().await;
                (sender_id.clone(), result)
            });
        }

        match futures.next().await {
            None => panic!("ran out of futures to poll in recv()"),
            Some(output) => {
                let (sender_id, result) = output;
                match result {
                    Ok(message) => {
                        tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, message);
                        Ok(message)
                    }
                    Err(_) => Err(MessageBusError::Disconnected(
                        sender_id,
                        receiver_id.clone(),
                    )),
                }
            }
        }
    }

    pub fn try_recv(&self, receiver: &impl Component) -> Result<M, MessageBusError> {
        let state = self
            .state
            .read()
            .expect("MessageBus state lock was poisoned in recv()");
        let receiver_id = receiver.id();
        let receivers = state
            .receivers
            .get(receiver_id)
            .ok_or(MessageBusError::NoSendersToReceiver(receiver_id.clone()))?;

        for (sender_id, rx) in receivers {
            match rx.try_recv() {
                Ok(message) => {
                    tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, message);
                    return Ok(message);
                }
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Closed) => {
                    return Err(MessageBusError::Disconnected(
                        sender_id.clone(),
                        receiver_id.clone(),
                    ))
                }
            }
        }

        Err(MessageBusError::NoMessagesAvailable(receiver_id.clone()))
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
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }
    }

    impl TestComponent {
        pub fn new(name: &str) -> Self {
            Self {
                id: ComponentId::new(name),
            }
        }
    }

    fn setup() -> ([TestComponent; 5], MessageBus<TestMessage>) {
        let bus = MessageBus::<TestMessage>::new("test bus");
        let components = [
            TestComponent::new("a"),
            TestComponent::new("b"),
            TestComponent::new("c"),
            TestComponent::new("d"),
            TestComponent::new("e"),
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

        let state = bus.state.read().unwrap();
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

        // Ensure that sending a message from `a` fails because it currently has no registered receivers.
        let a_msg = TestMessage {
            contents: String::from("message from a"),
        };
        assert_eq!(
            bus.send(&a, a_msg.clone()).await,
            Err(MessageBusError::NoReceiversForSender(a.id().clone()))
        );

        // Connect the five components to the bus such that:
        // `bus`
        //   ├─ `a`
        //   │   ├─ `b`
        //   │   ├─ `c`
        //   │
        //   ├─ `d`
        //       ├─ `e`
        bus.connect(&a, &b).expect("couldn't connect from a to b");
        bus.connect(&a, &c).expect("couldn't connect from a to c");
        bus.connect(&d, &e).expect("couldn't connect from d to e");

        // Ensure that `a` or `d` attempting to receive a message fails because they have no registered senders.
        assert_eq!(
            bus.recv(&a).await,
            Err(MessageBusError::NoSendersToReceiver(a.id().clone()))
        );
        assert_eq!(
            bus.recv(&d).await,
            Err(MessageBusError::NoSendersToReceiver(d.id().clone()))
        );

        // Ensure that a message sent from `a` is received by both `b` and `c`.
        bus.send(&a, a_msg.clone()).await.unwrap();
        let b_msg = bus.recv(&b).await.unwrap();
        assert_eq!(a_msg, b_msg);
        let c_msg = bus.recv(&c).await.unwrap();
        assert_eq!(a_msg, c_msg);

        // Ensure that a message sent from `a` is NOT received by `d` or `e`.
        assert_eq!(
            bus.try_recv(&d),
            Err(MessageBusError::NoSendersToReceiver(d.id().clone()))
        );
        assert_eq!(
            bus.try_recv(&e),
            Err(MessageBusError::NoMessagesAvailable(e.id().clone()))
        );
    }
}
