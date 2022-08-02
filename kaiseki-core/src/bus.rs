use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};

use crate::component::{Component, ComponentId};

#[derive(Error, Debug)]
pub enum BusError {
    #[error("disconnected from bus")]
    Disconnected,
    #[error("received unexpected message {0} on bus")]
    UnexpectedMessage(String),
}

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
        self.id.clone()
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
            id: ComponentId::new("Bus"),
            receivers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn send(&self, id: &ComponentId, message: T) -> Result<(), BusError> {
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

    pub async fn recv(&self, id: &ComponentId) -> Result<T, BusError> {
        let mut receivers = self.receivers.lock().await;
        let rx = receivers.get_mut(id).unwrap();
        if let Some(message) = rx.recv().await {
            return Ok(message);
        }
        Err(BusError::Disconnected)
    }

    pub async fn connect(&self, component_id: &ComponentId) -> Result<(), BusError> {
        let (tx_to_component, rx_from_bus) = mpsc::unbounded_channel();
        self.receivers
            .lock()
            .await
            .insert(component_id.clone(), rx_from_bus);
        self.senders
            .lock()
            .await
            .insert(component_id.clone(), tx_to_component);
        Ok(())
    }
}

impl<T: BusMessage> fmt::Debug for Bus<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus").finish()
    }
}
