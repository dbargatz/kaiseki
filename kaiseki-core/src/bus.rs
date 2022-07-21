use std::fmt;

use bytes::Bytes;
use crossbeam_channel::{bounded, Receiver, Sender};

use crate::component::Component;

#[derive(Debug)]
pub enum BusError {
    InvalidAddress,
}

pub type Result<T> = std::result::Result<T, BusError>;

#[derive(Debug)]
pub enum BusMessage {
    OscillatorTick { cycle: u64 },
    RaiseInterrupt,
    ReadAddress {
        address: usize,
        length: usize,
        response_channel: Sender<Bytes>,
    },
    WriteAddress {
        address: usize,
        data: Bytes,
        response_channel: Sender<Bytes>,
    },
}

#[derive(Debug)]
pub struct BusConnection {
    recv_from_bus: Receiver<BusMessage>,
    send_to_bus: Sender<BusMessage>,
}

impl BusConnection {
    pub fn new(rx: Receiver<BusMessage>, tx: Sender<BusMessage>) -> Self {
        BusConnection { recv_from_bus: rx, send_to_bus: tx }
    }

    pub fn tick(&self, cycle: u64) {
        let message = BusMessage::OscillatorTick { cycle };
        let _ = self.send_to_bus.send(message);
    }

    pub fn read(&self, address: usize, length: usize) -> Result<Bytes> {
        let (response_tx, response_rx) = bounded(1);
        let message = BusMessage::ReadAddress {
            address,
            length,
            response_channel: response_tx
        };
        let _ = self.send_to_bus.send(message);
        let result = response_rx.recv().unwrap();
        Ok(result)
    }

    pub fn read_u16(&self, address: usize) -> Result<u16> {
        let response = self.read(address, 2).unwrap();
        let value: u16 = ((response[0] as u16) << 8) as u16 | response[1] as u16;
        Ok(value)
    }

    pub fn recv(&self) -> Result<BusMessage> {
        let msg = self.recv_from_bus.recv().unwrap();
        Ok(msg)
    }

    pub fn write(&self, address: usize, data: Bytes) -> Result<Bytes> {
        let (response_tx, response_rx) = bounded(1);
        let message = BusMessage::WriteAddress {
            address,
            data,
            response_channel: response_tx
        };
        let _ = self.send_to_bus.send(message);
        let result = response_rx.recv().unwrap();
        Ok(result)
    }
}

pub struct Bus {
    memory_rx: Receiver<BusMessage>,
    memory_tx: Sender<BusMessage>,
}

impl Bus {
    pub fn new() -> Self {
        let (tx, rx) = bounded(100);
        Bus { memory_rx: rx, memory_tx: tx }
    }

    pub fn connect(&self, component: &mut impl Component) {
        let conn = BusConnection::new(self.memory_rx.clone(), self.memory_tx.clone());
        component.connect_to_bus(conn);
    }

    pub fn start(&self) {
        println!("Bus started.");
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus").finish()
    }
}
