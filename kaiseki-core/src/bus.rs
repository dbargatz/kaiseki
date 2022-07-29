use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::mpsc;

use crate::component::{Component, ComponentId};

#[derive(Debug)]
pub enum BusError {
    Disconnected,
}

pub type Result<T> = std::result::Result<T, BusError>;

pub trait BusMessage: 'static + Send + Sync + Clone + fmt::Debug {}

#[derive(Clone, Debug)]
struct Envelope<T: BusMessage> {
    sender_id: ComponentId,
    recipient_ids: Vec<ComponentId>,
    pub message: T,
}

#[derive(Debug)]
pub struct BusConnection<T: BusMessage> {
    id: ComponentId,
    recv_from_bus: mpsc::UnboundedReceiver<Envelope<T>>,
    send_to_bus: mpsc::UnboundedSender<Envelope<T>>,
}

impl<T: BusMessage> BusConnection<T> {
    fn new(
        id: ComponentId,
        tx: mpsc::UnboundedSender<Envelope<T>>,
        rx: mpsc::UnboundedReceiver<Envelope<T>>,
    ) -> Self {
        BusConnection {
            id,
            recv_from_bus: rx,
            send_to_bus: tx,
        }
    }

    pub fn blocking_recv(&mut self) -> Result<T> {
        match self.recv_from_bus.blocking_recv() {
            Some(envelope) => {
                tracing::trace!("{} received from bus: {:?}", self.id, envelope.message);
                Ok(envelope.message)
            }
            None => Err(BusError::Disconnected),
        }
    }

    pub fn blocking_send(&mut self, message: T) -> Result<()> {
        tracing::trace!("{} sending to bus: {:?}", self.id, message);
        let envelope = Envelope {
            sender_id: self.id,
            recipient_ids: Vec::new(),
            message,
        };
        if self.send_to_bus.send(envelope).is_err() {
            return Err(BusError::Disconnected);
        }
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<T> {
        match self.recv_from_bus.recv().await {
            Some(envelope) => {
                tracing::trace!("{} received from bus: {:?}", self.id, envelope.message);
                Ok(envelope.message)
            }
            None => Err(BusError::Disconnected),
        }
    }

    pub async fn send(&self, message: T) -> Result<()> {
        tracing::trace!("{} sending to bus: {:?}", self.id, message);
        let envelope = Envelope {
            sender_id: self.id,
            recipient_ids: Vec::new(),
            message,
        };
        if self.send_to_bus.send(envelope).is_err() {
            return Err(BusError::Disconnected);
        }
        Ok(())
    }
}

pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: HashMap<ComponentId, mpsc::UnboundedReceiver<Envelope<T>>>,
    senders: HashMap<ComponentId, mpsc::UnboundedSender<Envelope<T>>>,
}

#[async_trait]
impl<T: BusMessage> Component for Bus<T> {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        loop {
            if let Ok(envelope) = self.recv().await {
                self.send(envelope).await.unwrap();
            }
        }
    }
}

impl<T: BusMessage> Default for Bus<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BusMessage> Bus<T> {
    pub fn new() -> Self {
        Bus {
            id: ComponentId::new_v4(),
            receivers: HashMap::new(),
            senders: HashMap::new(),
        }
    }

    async fn send(&self, envelope: Envelope<T>) -> Result<()> {
        let mut receivers = envelope.recipient_ids;
        if receivers.is_empty() {
            receivers = Vec::new();
            for id in &self.senders {
                if *id.0 != envelope.sender_id {
                    receivers.push(*id.0);
                }
            }
        }

        for tx_id in receivers {
            let tx = &self.senders[&tx_id];
            let new_envelope = Envelope {
                sender_id: envelope.sender_id,
                recipient_ids: vec![tx_id],
                message: envelope.message.clone(),
            };
            tx.send(new_envelope.clone()).unwrap();
            tracing::trace!("{} => {}: {:?}", self.id, tx_id, new_envelope.message);
        }
        Ok(())
    }

    async fn recv(&mut self) -> Result<Envelope<T>> {
        let mut recv_futures = FuturesUnordered::new();
        for receiver in self.receivers.values_mut() {
            recv_futures.push(receiver.recv());
        }

        if let Some(Some(envelope)) = recv_futures.next().await {
            tracing::trace!(
                "{} => {}: {:?}",
                envelope.sender_id,
                self.id,
                envelope.message
            );
            return Ok(envelope);
        }

        Err(BusError::Disconnected)
    }

    pub fn connect(&mut self, component_id: &ComponentId) -> BusConnection<T> {
        let (tx_to_bus, rx_from_component) = mpsc::unbounded_channel();
        let (tx_to_component, rx_from_bus) = mpsc::unbounded_channel();
        self.receivers.insert(*component_id, rx_from_component);
        self.senders.insert(*component_id, tx_to_component);

        BusConnection::new(*component_id, tx_to_bus, rx_from_bus)
    }
}

impl<T: BusMessage> fmt::Debug for Bus<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus").finish()
    }
}
