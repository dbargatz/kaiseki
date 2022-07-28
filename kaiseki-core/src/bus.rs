use std::collections::HashMap;
use std::fmt;

use crossbeam_channel::{unbounded, Receiver, Select, Sender};

use crate::component::{Component, ComponentId};

#[derive(Debug)]
pub enum BusError {
    InvalidAddress,
}

pub type Result<T> = std::result::Result<T, BusError>;

pub trait BusMessage: Clone + fmt::Debug {}

#[derive(Clone, Debug)]
struct Envelope<T: BusMessage> {
    sender_id: ComponentId,
    recipient_ids: Vec<ComponentId>,
    pub message: T,
}

#[derive(Debug)]
pub struct BusConnection<T: BusMessage> {
    id: ComponentId,
    recv_from_bus: Receiver<Envelope<T>>,
    send_to_bus: Sender<Envelope<T>>,
}

impl<T: BusMessage> BusConnection<T> {
    fn new(id: ComponentId, tx: Sender<Envelope<T>>, rx: Receiver<Envelope<T>>) -> Self {
        BusConnection {
            id,
            recv_from_bus: rx,
            send_to_bus: tx,
        }
    }

    pub fn recv(&self) -> Result<T> {
        let envelope = self.recv_from_bus.recv().unwrap();
        tracing::trace!("received from bus: {:?}", envelope);
        Ok(envelope.message)
    }

    pub fn send(&self, message: T) {
        let envelope = Envelope {
            sender_id: self.id,
            recipient_ids: Vec::new(),
            message,
        };
        tracing::trace!("sending to bus: {:?}", envelope);
        self.send_to_bus.send(envelope).unwrap();
    }
}

pub struct Bus<T: BusMessage> {
    id: ComponentId,
    receivers: HashMap<ComponentId, Receiver<Envelope<T>>>,
    senders: HashMap<ComponentId, Sender<Envelope<T>>>,
}

impl<T: BusMessage> Component for Bus<T> {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&mut self) {
        loop {
            let envelope = self.recv();
            self.send(envelope);
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

    fn send(&self, envelope: Envelope<T>) {
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
            let new_envelope = Envelope { sender_id: envelope.sender_id, recipient_ids: vec![tx_id], message: envelope.message.clone() };
            tx.send(new_envelope.clone()).unwrap();
            tracing::trace!("bus sent {:?} to {}", new_envelope.message, tx_id);
        }
    }

    fn recv(&self) -> Envelope<T> {
        let receivers: Vec<&Receiver<Envelope<T>>> = self.receivers.values().collect();
        let mut sel = Select::new();
        for rx in &receivers {
            sel.recv(rx);
        }
        let oper = sel.select();
        let index = oper.index();
        let envelope = oper.recv(receivers[index]).unwrap();
        tracing::trace!("bus received {:?} from {}", envelope.message, envelope.sender_id);
        envelope
    }

    pub fn connect(&mut self, component_id: &ComponentId) -> BusConnection<T> {
        let (tx_to_bus, rx_from_component) = unbounded();
        let (tx_to_component, rx_from_bus) = unbounded();
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
