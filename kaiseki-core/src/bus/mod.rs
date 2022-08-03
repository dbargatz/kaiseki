use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use async_channel::{unbounded, Receiver, Sender};
use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::Mutex;

pub mod message;

use crate::component::{Component, ComponentId};

pub trait BusMessage: 'static + Send + Sync + Clone + fmt::Debug {}

#[derive(Error, Debug)]
pub enum BusError {
    #[error("disconnected from bus")]
    Disconnected,
    #[error("received unexpected message {0} on bus")]
    UnexpectedMessage(String),
}


#[derive(Clone)]
pub struct Envelope<T: BusMessage> {
    from: ComponentId,
    to: ComponentId,
    message: T,
}

impl<T: BusMessage> Envelope<T> {
    pub fn new(from: &ComponentId, to: &ComponentId, message: T) -> Self {
        Envelope {
            from: from.clone(),
            to: to.clone(),
            message,
        }
    }
}

impl<T: BusMessage> fmt::Debug for Envelope<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} => {}: {:?}", self.from, self.to, self.message)
    }
}

#[derive(Clone)]
pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: Arc<Mutex<HashMap<ComponentId, Receiver<Envelope<T>>>>>,
    senders: Arc<Mutex<HashMap<ComponentId, Sender<Envelope<T>>>>>,
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

    pub async fn broadcast(&self, id: &ComponentId, message: T) -> Result<(), BusError> {
        let senders = self.senders.lock().await;
        for (tx_id, tx) in senders.iter() {
            if *tx_id == *id {
                continue;
            }

            let envelope = Envelope::new(id, tx_id, message.clone());
            tracing::trace!("{} send: {:?}", id, envelope);
            tx.send(envelope).await.unwrap();
        }
        Ok(())
    }

    pub async fn send(
        &self,
        from: &ComponentId,
        to: &ComponentId,
        message: T,
    ) -> Result<(), BusError> {
        let senders = self.senders.lock().await;
        let tx = senders.get(to).unwrap();

        let envelope = Envelope::new(from, to, message);
        tracing::trace!("{} send: {:?}", from, envelope);
        tx.send(envelope).await.unwrap();
        Ok(())
    }

    pub async fn recv(&self, id: &ComponentId) -> Result<(ComponentId, T), BusError> {
        let mut receivers = self.receivers.lock().await;
        let rx = receivers.get_mut(id).unwrap().clone();
        drop(receivers);

        loop {
            tracing::trace!("{} entering recv() wait", id);
            if let Ok(envelope) = rx.recv().await {
                tracing::trace!("{} recv: {:?}", id, envelope);
                if envelope.to == *id {
                    return Ok((envelope.from, envelope.message));
                }
            }
        }
    }

    pub async fn connect(&self, component_id: &ComponentId) -> Result<(), BusError> {
        let (tx_to_component, rx_from_bus) = unbounded();
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
