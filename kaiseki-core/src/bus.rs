use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex};

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
    pub message: T,
}

#[derive(Clone)]
pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: Arc<Mutex<HashMap<ComponentId, mpsc::UnboundedReceiver<Envelope<T>>>>>,
    senders: Arc<Mutex<HashMap<ComponentId, mpsc::UnboundedSender<Envelope<T>>>>>,
}

#[async_trait]
impl<T: BusMessage> Component for Bus<T> {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {}
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
            receivers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn recv_direct(&self, id: &ComponentId) -> Result<T> {
        Ok(self.recv(id).await.unwrap().message)
    }

    pub async fn send_direct(&self, id: &ComponentId, message: T) -> Result<()> {
        let new_envelope = Envelope {
            sender_id: *id,
            message: message.clone(),
        };
        self.send(new_envelope).await
    }

    async fn send(&self, envelope: Envelope<T>) -> Result<()> {
        let senders = self.senders.lock().await;
        for (tx_id, tx) in senders.iter() {
            if *tx_id == envelope.sender_id {
                continue;
            }

            let new_envelope = Envelope {
                sender_id: envelope.sender_id,
                message: envelope.message.clone(),
            };
            tracing::trace!("{} => {}: {:?}", self.id, tx_id, new_envelope.message);
            tx.send(new_envelope).unwrap();
        }
        Ok(())
    }

    async fn recv(&self, id: &ComponentId) -> Result<Envelope<T>> {
        let mut receivers = self.receivers.lock().await;
        let rx = receivers.get_mut(id).unwrap();
        if let Some(envelope) = rx.recv().await {
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

    pub async fn connect(&self, component_id: &ComponentId) -> Result<()> {
        let (tx_to_component, rx_from_bus) = mpsc::unbounded_channel();
        self.receivers
            .lock()
            .await
            .insert(*component_id, rx_from_bus);
        self.senders
            .lock()
            .await
            .insert(*component_id, tx_to_component);
        Ok(())
    }
}

impl<T: BusMessage> fmt::Debug for Bus<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus").finish()
    }
}
