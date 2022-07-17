use bytes::Bytes;
use std::fmt;
use tokio::sync::{mpsc, oneshot};
use crate::component::Component;

pub enum SystemBusError {
    InvalidAddress,
}

pub type Result<T> = std::result::Result<T, SystemBusError>;

pub enum MemoryMessage {
    Read {
        address: usize,
        length: usize,
        response_channel: oneshot::Sender<Bytes>,
    },
    Write {
        address: usize,
        data: Bytes,
        response_channel: oneshot::Sender<Bytes>,
    },
}

pub struct SystemBus {
    memory_rx: mpsc::Receiver<MemoryMessage>,
    memory_tx: mpsc::Sender<MemoryMessage>,
}

impl SystemBus {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(16);
        SystemBus { memory_rx: rx, memory_tx: tx }
    }

    pub fn connect(&self, other: &impl Component) {
        
    }

    pub async fn read(&self, address: usize, length: usize) -> Result<Bytes> {
        let (response_tx, response_rx) = oneshot::channel();
        let message = MemoryMessage::Read {
            address,
            length,
            response_channel: response_tx
        };
        let _ = self.memory_tx.send(message);
        let result = response_rx.await.unwrap();
        Ok(result)
    }

    pub async fn read_u16(&self, address: usize) -> Result<u16> {
        let response = self.read(address, 2).await?;
        let value: u16 = (response[0] << 8) as u16 | response[1] as u16;
        Ok(value)
    }
}

impl Component for SystemBus {}

impl fmt::Debug for SystemBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemBus").finish()
    }
}
