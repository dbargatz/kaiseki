use std::collections::HashMap;

use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use thiserror::Error;
use tokio::time::Instant;

use crate::component::{Component2, Component2Id, Component2Ref};

#[derive(Debug, Error)]
pub enum BaseBusError<T: BaseBusMessage> {
    #[error("component {0} is already connected, cannot connect")]
    AlreadyConnected(Component2Id),
    #[error("component {0} is not connected, cannot disconnect")]
    AlreadyDisconnected(Component2Id),
    #[error("could not send broadcast message from component {from} to component {to}")]
    BroadcastFailure {
        from: Component2Id,
        to: Component2Id,
        source: async_channel::SendError<BaseBusControlMessage<T>>,
    },
}

pub type Result<T, U> = std::result::Result<T, BaseBusError<U>>;

pub trait BaseBusMessage: 'static + Clone + core::fmt::Debug + Send {}

#[derive(Clone, Debug)]
pub enum BaseBusControlMessage<T: BaseBusMessage> {
    Broadcast {
        sender_id: Component2Id,
        message: T,
    },
    Connect {
        sender_id: Component2Id,
        response_sender: async_channel::Sender<BaseBusRef<T>>,
    },
    Disconnect {
        sender_id: Component2Id,
        response_sender: async_channel::Sender<Result<(), T>>,
    },
}

impl<T: BaseBusMessage> BaseBusMessage for BaseBusControlMessage<T> {}

type ChannelPair<T> = (
    async_channel::Sender<BaseBusControlMessage<T>>,
    async_channel::Receiver<BaseBusControlMessage<T>>,
);

#[derive(Debug)]
pub struct BaseBus<T: BaseBusMessage> {
    id: Component2Id,
    clients: HashMap<Component2Id, ChannelPair<T>>,
    owner_receiver: async_channel::Receiver<BaseBusControlMessage<T>>,
}

#[async_trait]
impl<T: BaseBusMessage> Component2 for BaseBus<T> {
    type Ref = BaseBusRef<T>;

    fn id(&self) -> &Component2Id {
        &self.id
    }

    async fn run(&mut self) {
        loop {
            // Remove any connected clients with closed receivers before entering the listen phase.
            let dead_clients: Vec<Component2Id> = self
                .clients
                .iter()
                .filter(|(_, (_, r))| r.is_closed())
                .map(|(i, _)| *i)
                .collect();
            for id in dead_clients {
                tracing::info!("removing dead client {}", id);
                self.disconnect(&id).unwrap();
            }

            // Build up the list of live, connected client receivers for use with select!.
            let mut client_recvs = FuturesUnordered::new();
            let mut live_recvs: Vec<_> =
                self.clients.iter().map(|(_, (_, rx))| rx.clone()).collect();

            if !self.owner_receiver.is_closed() {
                live_recvs.push(self.owner_receiver.clone());
            }

            for rx in live_recvs {
                client_recvs.push(async move { rx.recv().await });
            }

            tokio::select! {
                Some(result) = client_recvs.next() => {
                    match result {
                        Err(err) => tracing::info!("component disconnected: {}", err),
                        Ok(BaseBusControlMessage::Broadcast { sender_id, message }) => {
                            self.broadcast(&sender_id, message).await.unwrap();
                        },
                        Ok(BaseBusControlMessage::Connect { sender_id, response_sender }) => {
                            tracing::info!("connecting component {} to bus", sender_id);
                            let response = self.connect(&sender_id).unwrap();
                            response_sender.send(response).await.unwrap();
                        },
                        Ok(BaseBusControlMessage::Disconnect { sender_id, response_sender }) => {
                            tracing::info!("disconnecting component {} from bus", sender_id);
                            let response = self.disconnect(&sender_id);
                            response_sender.send(response).await.unwrap();
                        },
                    };
                },
                else => {
                    tracing::info!("bus exiting");
                    break;
                },
            }
        }
    }
}

impl<T: BaseBusMessage> BaseBus<T> {
    pub fn create(id: Component2Id) -> BaseBusRef<T> {
        // TODO: how should this channel be bounded?
        let (owner_sender, owner_receiver) = async_channel::bounded(8);
        let mut bus = Self {
            id,
            clients: HashMap::new(),
            owner_receiver,
        };
        tokio::spawn(async move { bus.run().await });
        <BaseBus<T> as Component2>::Ref::new(&id, owner_sender)
    }

    pub async fn broadcast(&self, sender_id: &Component2Id, msg: T) -> Result<(), T> {
        let mut result = Ok(());
        let send_start = Instant::now();
        for (id, (sender, _)) in self.clients.iter() {
            if id == sender_id {
                continue;
            }

            let envelope = BaseBusControlMessage::Broadcast {
                sender_id: *sender_id,
                message: msg.clone(),
            };
            if let Err(err) = sender.send(envelope).await {
                if result.is_ok() {
                    let new_err = BaseBusError::BroadcastFailure {
                        from: *sender_id,
                        to: *id,
                        source: err,
                    };
                    result = Err(new_err);
                }
            };
        }
        let send_duration = Instant::now() - send_start;
        tracing::info!(
            "broadcast took {}ns to send to {} client(s)",
            send_duration.as_nanos(),
            self.clients.len()
        );
        result
    }

    pub fn connect(&mut self, component_id: &Component2Id) -> Result<BaseBusRef<T>, T> {
        if self.clients.contains_key(component_id) {
            return Err(BaseBusError::AlreadyConnected(*component_id));
        }

        // TODO: how should this channel be bounded?
        let (ref_sender, ref_receiver) = async_channel::bounded(8);
        self.clients
            .insert(*component_id, (ref_sender.clone(), ref_receiver));
        Ok(BaseBusRef::new(component_id, ref_sender))
    }

    pub fn disconnect(&mut self, component_id: &Component2Id) -> Result<(), T> {
        match self.clients.remove(component_id) {
            Some(_) => {
                tracing::info!("component {} was disconnected from the bus", component_id);
                Ok(())
            }
            None => Err(BaseBusError::AlreadyDisconnected(*component_id)),
        }
    }
}

pub type BaseBusRef<T> = Component2Ref<BaseBusControlMessage<T>>;

impl<T: BaseBusMessage> BaseBusRef<T> {
    pub async fn broadcast(&self, msg: T) {
        let envelope = BaseBusControlMessage::Broadcast {
            sender_id: *self.id(),
            message: msg,
        };
        self.send(envelope).await;
    }

    pub fn connect(&self, client_id: Component2Id) -> BaseBusRef<T> {
        let (response_sender, response_receiver) = async_channel::bounded(1);
        let msg = BaseBusControlMessage::Connect {
            sender_id: client_id,
            response_sender,
        };
        self.send_blocking(msg);
        response_receiver.recv_blocking().unwrap()
    }

    // TODO: needs to await positive response to disconnect from bus
    pub async fn disconnect(&self) {
        let (response_sender, response_receiver) = async_channel::bounded(1);
        let msg = BaseBusControlMessage::Disconnect {
            sender_id: *self.id(),
            response_sender,
        };
        self.send(msg).await;
        response_receiver.recv().await.unwrap().unwrap()
    }
}
