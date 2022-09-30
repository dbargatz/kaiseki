use futures::{channel::oneshot, stream::FuturesUnordered, StreamExt};
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

struct MessageEnvelope<M: BusMessage> {
    pub response_tx: Option<oneshot::Sender<M>>,
    pub request: M,
}

type ReceiverList<M> = Vec<(ComponentId, Receiver<MessageEnvelope<M>>)>;
type SenderList<M> = Vec<(ComponentId, Sender<MessageEnvelope<M>>)>;

struct MessageBusState<M: BusMessage> {
    receivers: HashMap<ComponentId, ReceiverList<M>>,
    senders: HashMap<ComponentId, SenderList<M>>,
}

#[derive(Clone)]
pub struct MessageBusConnection<M: BusMessage> {
    bus: MessageBus<M>,
    component_id: ComponentId,
}

impl<M: BusMessage> fmt::Debug for MessageBusConnection<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageBusConnection[{}]", self.component_id)
    }
}

impl<M: BusMessage> MessageBusConnection<M> {
    pub async fn recv(&self) -> Result<(M, Option<oneshot::Sender<M>>), MessageBusError> {
        self.bus.recv(&self.component_id).await
    }

    pub async fn request(&self, request: M) -> Result<M, MessageBusError> {
        self.bus.request(&self.component_id, request).await
    }

    pub async fn send(&self, message: M) -> Result<(), MessageBusError> {
        self.bus.send(&self.component_id, message).await
    }

    pub fn try_recv(&self) -> Result<(M, Option<oneshot::Sender<M>>), MessageBusError> {
        self.bus.try_recv(&self.component_id)
    }
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

impl<M: BusMessage> fmt::Debug for MessageBus<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MessageBus[{}]", self.id)
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

    pub fn connect(
        &self,
        sender_id: &ComponentId,
        receiver_id: &ComponentId,
    ) -> Result<(MessageBusConnection<M>, MessageBusConnection<M>)> {
        let (tx_sender_to_receiver, rx_receiver_from_sender) = async_channel::unbounded();

        let mut state = self.state.write().unwrap();
        let receiver_entry = state.receivers.entry(receiver_id.clone()).or_default();
        receiver_entry.push((sender_id.clone(), rx_receiver_from_sender));
        let sender_entry = state.senders.entry(sender_id.clone()).or_default();
        sender_entry.push((receiver_id.clone(), tx_sender_to_receiver));

        let sender_connection = MessageBusConnection::<M> {
            bus: self.clone(),
            component_id: sender_id.clone(),
        };
        let receiver_connection = MessageBusConnection::<M> {
            bus: self.clone(),
            component_id: receiver_id.clone(),
        };
        Ok((sender_connection, receiver_connection))
    }

    pub async fn recv(
        &self,
        receiver_id: &ComponentId,
    ) -> Result<(M, Option<oneshot::Sender<M>>), MessageBusError> {
        let receivers;
        {
            let state = self.state.read().unwrap();
            receivers = state
                .receivers
                .get(receiver_id)
                .ok_or_else(|| MessageBusError::NoSendersToReceiver(receiver_id.clone()))?
                .clone();
        }

        let mut futures = FuturesUnordered::new();
        for (sender_id, rx) in receivers {
            futures.push(async move {
                let result = rx.recv().await;
                (sender_id, result)
            });
        }

        match futures.next().await {
            None => panic!("ran out of futures to poll in recv()"),
            Some(output) => {
                let (sender_id, result) = output;
                match result {
                    Ok(message) => {
                        tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, message.request);
                        Ok((message.request, message.response_tx))
                    }
                    Err(_) => Err(MessageBusError::Disconnected(
                        sender_id,
                        receiver_id.clone(),
                    )),
                }
            }
        }
    }

    pub async fn request(&self, sender_id: &ComponentId, request: M) -> Result<M, MessageBusError> {
        let senders;
        {
            let state = self.state.read().unwrap();
            senders = state
                .senders
                .get(sender_id)
                .ok_or_else(|| MessageBusError::NoReceiversForSender(sender_id.clone()))?
                .clone();
        }

        let mut futures = FuturesUnordered::new();

        for (receiver_id, tx) in senders {
            let (responder_tx, responder_rx) = oneshot::channel();
            let envelope = MessageEnvelope::<M> {
                response_tx: Some(responder_tx),
                request: request.clone(),
            };
            match tx.send(envelope).await {
                Ok(_) => tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, request),
                Err(_) => {
                    return Err(MessageBusError::Disconnected(
                        sender_id.clone(),
                        receiver_id.clone(),
                    ))
                }
            }
            futures.push(async move {
                let response = responder_rx.await;
                (receiver_id.clone(), response)
            });
        }

        match futures.next().await {
            None => panic!("ran out of futures to poll in request()"),
            Some(output) => {
                let (receiver_id, result) = output;
                match result {
                    Ok(message) => {
                        tracing::trace!("{} <= {}: {:?}", sender_id, receiver_id, message);
                        Ok(message)
                    }
                    Err(_) => Err(MessageBusError::Disconnected(
                        sender_id.clone(),
                        receiver_id,
                    )),
                }
            }
        }
    }

    pub async fn send(&self, sender_id: &ComponentId, message: M) -> Result<(), MessageBusError> {
        let senders;
        {
            let state = self.state.read().unwrap();
            senders = state
                .senders
                .get(sender_id)
                .ok_or_else(|| MessageBusError::NoReceiversForSender(sender_id.clone()))?
                .clone();
        }

        for (receiver_id, tx) in senders {
            let envelope = MessageEnvelope::<M> {
                response_tx: None,
                request: message.clone(),
            };
            match tx.send(envelope).await {
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

    pub fn try_recv(
        &self,
        receiver_id: &ComponentId,
    ) -> Result<(M, Option<oneshot::Sender<M>>), MessageBusError> {
        let receivers;
        {
            let state = self.state.read().unwrap();
            receivers = state
                .receivers
                .get(receiver_id)
                .ok_or_else(|| MessageBusError::NoSendersToReceiver(receiver_id.clone()))?
                .clone();
        }

        for (sender_id, rx) in receivers {
            match rx.try_recv() {
                Ok(message) => {
                    tracing::trace!("{} => {}: {:?}", sender_id, receiver_id, message.request);
                    return Ok((message.request, message.response_tx));
                }
                Err(TryRecvError::Empty) => continue,
                Err(TryRecvError::Closed) => {
                    return Err(MessageBusError::Disconnected(
                        sender_id,
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
        let (a_conn, b_conn) = bus
            .connect(a.id(), b.id())
            .expect("couldn't connect from a to b");
        assert_eq!(&a_conn.component_id, a.id());
        assert_eq!(&b_conn.component_id, b.id());

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
    async fn send_recv_try_recv_works() {
        let ([a, b, c, d, e], bus) = setup();

        // Ensure that sending a message from `a` fails because it currently has no registered receivers.
        let a_msg = TestMessage {
            contents: String::from("message from a"),
        };
        assert_eq!(
            bus.send(a.id(), a_msg.clone()).await,
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
        bus.connect(a.id(), b.id())
            .expect("couldn't connect from a to b");
        bus.connect(a.id(), c.id())
            .expect("couldn't connect from a to c");
        bus.connect(d.id(), e.id())
            .expect("couldn't connect from d to e");

        // Ensure that `a` or `d` attempting to receive a message fails because they have no registered senders.
        assert_eq!(
            bus.recv(a.id()).await.unwrap_err(),
            MessageBusError::NoSendersToReceiver(a.id().clone())
        );
        assert_eq!(
            bus.recv(d.id()).await.unwrap_err(),
            MessageBusError::NoSendersToReceiver(d.id().clone())
        );

        // Ensure that a message sent from `a` is received by both `b` and `c`.
        bus.send(a.id(), a_msg.clone()).await.unwrap();
        let (b_msg, _) = bus.recv(b.id()).await.unwrap();
        assert_eq!(a_msg, b_msg);
        let (c_msg, _) = bus.recv(c.id()).await.unwrap();
        assert_eq!(a_msg, c_msg);

        // Ensure that a message sent from `a` is NOT received by `d` or `e`.
        assert_eq!(
            bus.try_recv(d.id()).unwrap_err(),
            MessageBusError::NoSendersToReceiver(d.id().clone())
        );
        assert_eq!(
            bus.try_recv(e.id()).unwrap_err(),
            MessageBusError::NoMessagesAvailable(e.id().clone())
        );
    }

    #[tokio::test]
    async fn message_bus_connection_works() {
        let ([a, b, c, d, e], bus) = setup();

        // Connect the five components to the bus such that:
        // `bus`
        //   ├─ `a`
        //   │   ├─ `b`
        //   │   ├─ `c`
        //   │
        //   ├─ `d`
        //       ├─ `e`
        let (a_conn, b_conn) = bus
            .connect(a.id(), b.id())
            .expect("couldn't connect from a to b");
        let (_, c_conn) = bus
            .connect(a.id(), c.id())
            .expect("couldn't connect from a to c");
        let (d_conn, e_conn) = bus
            .connect(d.id(), e.id())
            .expect("couldn't connect from d to e");

        // Ensure that `a_conn` or `d_conn` attempting to receive a message fails because they have no registered senders.
        assert_eq!(
            a_conn.recv().await.unwrap_err(),
            MessageBusError::NoSendersToReceiver(a.id().clone())
        );
        assert_eq!(
            d_conn.recv().await.unwrap_err(),
            MessageBusError::NoSendersToReceiver(d.id().clone())
        );

        // Ensure that a message sent from `a_conn` is received by both `b_conn` and `c_conn`.
        let a_msg = TestMessage {
            contents: String::from("message from a"),
        };
        a_conn.send(a_msg.clone()).await.unwrap();
        let (b_msg, _) = b_conn.recv().await.unwrap();
        assert_eq!(a_msg, b_msg);
        let (c_msg, _) = c_conn.recv().await.unwrap();
        assert_eq!(a_msg, c_msg);

        // Ensure that a message sent from `a_conn` is NOT received by `d_conn` or `e_conn`.
        assert_eq!(
            d_conn.try_recv().unwrap_err(),
            MessageBusError::NoSendersToReceiver(d.id().clone())
        );
        assert_eq!(
            e_conn.try_recv().unwrap_err(),
            MessageBusError::NoMessagesAvailable(e.id().clone())
        );
    }

    #[tokio::test]
    async fn request_works() {
        let ([a, b, c, _, _], bus) = setup();

        // Connect the three components to the bus such that:
        // `bus`
        //   ├─ `a`
        //   │   ├─ `b`
        //   │   ├─ `c`
        let (a_conn, b_conn) = bus
            .connect(a.id(), b.id())
            .expect("couldn't connect from a to b");
        let (_, c_conn) = bus
            .connect(a.id(), c.id())
            .expect("couldn't connect from a to c");

        // Ensure that a request sent from `a_conn` can be responded to by `b_conn`.
        let a_request = TestMessage {
            contents: String::from("request from a"),
        };
        let a_task_conn = a_conn.clone();
        let a_task_request = a_request.clone();
        let a_task =
            tokio::spawn(async move { a_task_conn.request(a_task_request).await.unwrap() });

        let (b_request, b_responder) = b_conn.recv().await.unwrap();
        assert_eq!(a_request.clone(), b_request);
        let b_responder = b_responder.unwrap();
        assert!(!b_responder.is_canceled());
        let b_response = TestMessage {
            contents: String::from("response from b"),
        };
        b_responder.send(b_response.clone()).unwrap();

        let a_response = a_task.await.unwrap();
        assert_eq!(a_response, b_response);

        // Ensure that `c_conn` sees the same request from `a_conn`, but also sees the
        // responder provided by `a_conn` as cancelled/dropped.
        let (c_request, c_responder) = c_conn.recv().await.unwrap();
        assert_eq!(a_request.clone(), c_request);
        assert!(c_responder.unwrap().is_canceled());
    }
}
