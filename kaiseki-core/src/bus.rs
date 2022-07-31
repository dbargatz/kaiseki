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

#[derive(Clone)]
pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: Arc<Mutex<HashMap<ComponentId, mpsc::UnboundedReceiver<T>>>>,
    senders: Arc<Mutex<HashMap<ComponentId, mpsc::UnboundedSender<T>>>>,
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

    pub async fn send(&self, id: &ComponentId, message: T) -> Result<()> {
        let senders = self.senders.lock().await;
        for (tx_id, tx) in senders.iter() {
            if *tx_id == *id {
                continue;
            }

            tracing::trace!("{} => {}: {:?}", id, tx_id, message);
            tx.send(message.clone()).unwrap();
        }
        Ok(())
    }

    pub async fn recv(&self, id: &ComponentId) -> Result<T> {
        let mut receivers = self.receivers.lock().await;
        let rx = receivers.get_mut(id).unwrap();
        if let Some(message) = rx.recv().await {
            tracing::trace!("recv => {}: {:?}", id, message);
            return Ok(message);
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
