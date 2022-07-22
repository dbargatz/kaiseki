use std::fmt;
use std::sync::mpsc;

use bytes::Bytes;
use crossbeam_channel::{bounded, Sender};

use crate::component::Component;

#[derive(Debug)]
pub enum BusError {
    InvalidAddress,
}

pub type Result<T> = std::result::Result<T, BusError>;

#[derive(Clone, Debug)]
pub enum BusMessage {
    OscillatorTick {
        cycle: u64,
    },
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
    recv_from_bus: bus::BusReader<BusMessage>,
    send_to_bus: mpsc::SyncSender<BusMessage>,
}

impl BusConnection {
    pub fn new(rx: bus::BusReader<BusMessage>, tx: mpsc::SyncSender<BusMessage>) -> Self {
        BusConnection {
            recv_from_bus: rx,
            send_to_bus: tx,
        }
    }

    pub fn tick(&mut self, cycle: u64) {
        let message = BusMessage::OscillatorTick { cycle };
        let _ = self.send(message);
    }

    pub fn read(&mut self, address: usize, length: usize) -> Result<Bytes> {
        let (response_tx, response_rx) = bounded(1);
        let message = BusMessage::ReadAddress {
            address,
            length,
            response_channel: response_tx,
        };
        let _ = self.send(message);
        let result = response_rx.recv().unwrap();
        Ok(result)
    }

    pub fn read_u16(&mut self, address: usize) -> Result<u16> {
        let response = self.read(address, 2).unwrap();
        let value: u16 = ((response[0] as u16) << 8) as u16 | response[1] as u16;
        Ok(value)
    }

    pub fn recv(&mut self) -> Result<BusMessage> {
        let msg = self.recv_from_bus.recv().unwrap();
        Ok(msg)
    }

    pub fn send(&mut self, message: BusMessage) {
        let _ = self.send_to_bus.send(message);
    }

    pub fn write(&mut self, address: usize, data: Bytes) -> Result<Bytes> {
        let (response_tx, response_rx) = bounded(1);
        let message = BusMessage::WriteAddress {
            address,
            data,
            response_channel: response_tx,
        };
        let _ = self.send(message);
        let result = response_rx.recv().unwrap();
        Ok(result)
    }
}

pub struct Bus {
    rx: mpsc::Receiver<BusMessage>,
    tx: mpsc::SyncSender<BusMessage>,
    bus: bus::Bus<BusMessage>,
}

impl Component for Bus {
    fn connect_to_bus(&mut self, _bus: BusConnection) {
        println!("can't connect bus to bus yet");
    }

    fn start(&mut self) {
        loop {
            for msg in self.rx.iter() {
                self.bus.broadcast(msg);
            }
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::sync_channel(100);
        let bus = bus::Bus::<BusMessage>::new(1);
        Bus { rx, tx, bus }
    }

    pub fn connect(&mut self, component: &mut impl Component) {
        let rx = self.bus.add_rx();
        let conn = BusConnection::new(rx, self.tx.clone());
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
