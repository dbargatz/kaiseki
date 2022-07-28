use std::collections::HashMap;
use std::fmt;

use crossbeam_channel::{unbounded, Receiver, Select, Sender};

use crate::component::{Component, ComponentId};

#[derive(Debug)]
pub enum BusError {
    InvalidAddress,
}

pub type Result<T> = std::result::Result<T, BusError>;

pub trait BusMessage: Clone + fmt::Debug {
    fn sender(&self) -> ComponentId;
    fn recipients(&self) -> Vec<ComponentId>;
}

#[derive(Clone, Debug)]
pub struct BusMessageMetadata {
    pub sender: ComponentId,
    pub recipients: Vec<ComponentId>,
}

#[derive(Debug)]
pub struct BusConnection<T: BusMessage> {
    recv_from_bus: Receiver<T>,
    send_to_bus: Sender<T>,
}

impl<T: BusMessage> BusConnection<T> {
    pub fn new(rx: Receiver<T>, tx: Sender<T>) -> Self {
        BusConnection {
            recv_from_bus: rx,
            send_to_bus: tx,
        }
    }

    pub fn recv(&mut self) -> Result<T> {
        let msg = self.recv_from_bus.recv().unwrap();
        tracing::trace!("received from bus: {:?}", msg);
        Ok(msg)
    }

    pub fn send(&mut self, message: T) {
        tracing::trace!("sending to bus: {:?}", message);
        self.send_to_bus.send(message).unwrap();
    }
}

pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: HashMap<ComponentId, Receiver<T>>,
    senders: HashMap<ComponentId, Sender<T>>,
}

impl<T: BusMessage> Component for Bus<T> {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&mut self) {
        loop {
            let msg = self.recv().unwrap();
            self.send(msg, &[]);
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

    fn send(&self, message: T, receiver_ids: &[ComponentId]) {
        let mut receivers = Vec::from(receiver_ids);
        let sender = message.sender();
        if receiver_ids.is_empty() {
            receivers = Vec::new();
            for id in &self.senders {
                if *id.0 != sender {
                    receivers.push(*id.0);
                }
            }
        }

        for tx_id in receivers {
            let tx = &self.senders[&tx_id];
            tx.send(message.clone()).unwrap();
            tracing::trace!("bus sent {:?} to {}", message, tx_id);
        }
    }

    fn recv(&self) -> Result<T> {
        let receivers: Vec<&Receiver<T>> = self.receivers.values().collect();
        let mut sel = Select::new();
        for rx in &receivers {
            sel.recv(rx);
        }
        let oper = sel.select();
        let index = oper.index();
        let message = oper.recv(receivers[index]).unwrap();
        tracing::trace!("bus received {:?}", message);
        Ok(message)
    }

    pub fn connect(&mut self, component_id: &ComponentId) -> BusConnection<T> {
        let (tx_to_bus, rx_from_component) = unbounded();
        let (tx_to_component, rx_from_bus) = unbounded();
        self.receivers.insert(*component_id, rx_from_component);
        self.senders.insert(*component_id, tx_to_component);

        BusConnection::new(rx_from_bus, tx_to_bus)
    }
}

impl<T: BusMessage> fmt::Debug for Bus<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus").finish()
    }
}
