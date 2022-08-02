use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

use crate::bus::{Bus, BusError, BusMessage};
use crate::component::{Component, ComponentId};

#[derive(Clone, Debug)]
pub enum MemoryBusMessage {
    ReadAddress { address: usize, length: usize },
    ReadResponse { data: Bytes },
    WriteAddress { address: usize, data: Bytes },
    WriteResponse,
}

impl BusMessage for MemoryBusMessage {}

pub type MemoryBus = Bus<MemoryBusMessage>;

impl MemoryBus {
    pub async fn read(
        &self,
        id: &ComponentId,
        address: usize,
        length: usize,
    ) -> Result<Bytes, BusError> {
        let request = MemoryBusMessage::ReadAddress { address, length };
        self.send(id, request).await.unwrap();
        let response = self.recv(id).await.unwrap();

        if let MemoryBusMessage::ReadResponse { data } = response {
            Ok(data)
        } else {
            tracing::warn!(
                "unexpected response to ReadAddress on memory bus: {:?}",
                response
            );
            let msg_str = format!("{:?}", response);
            Err(BusError::UnexpectedMessage(msg_str))
        }
    }

    pub async fn write(
        &self,
        id: &ComponentId,
        address: usize,
        data: Bytes,
    ) -> Result<(), BusError> {
        let request = MemoryBusMessage::WriteAddress { address, data };
        self.send(id, request).await.unwrap();
        let response = self.recv(id).await.unwrap();

        if let MemoryBusMessage::WriteResponse = response {
            Ok(())
        } else {
            tracing::warn!(
                "unexpected response to WriteAddress on memory bus: {:?}",
                response
            );
            let msg_str = format!("{:?}", response);
            Err(BusError::UnexpectedMessage(msg_str))
        }
    }
}

#[derive(Debug)]
pub struct RAM<const N: usize> {
    id: ComponentId,
    bus: MemoryBus,
    memory: [u8; N],
}

#[async_trait]
impl<const N: usize> Component for RAM<N> {
    fn id(&self) -> ComponentId {
        self.id.clone()
    }

    async fn start(&mut self) {
        loop {
            if let Ok(MemoryBusMessage::ReadAddress { address, length }) =
                self.bus.recv(&self.id).await
            {
                tracing::trace!("read request: {} bytes at 0x{:X}", length, address);
                let mem = Bytes::copy_from_slice(&self.memory[address..address + length]);
                let response = MemoryBusMessage::ReadResponse { data: mem };
                self.bus.send(&self.id, response).await.unwrap();
            }
        }
    }
}

impl<const N: usize> RAM<N> {
    pub fn new(memory_bus: &MemoryBus) -> Self {
        let id = ComponentId::new("RAM");
        RAM {
            id,
            bus: memory_bus.clone(),
            memory: [0; N],
        }
    }

    pub fn read(&self, addr: usize, len: usize) -> Bytes {
        Bytes::copy_from_slice(&self.memory[addr..addr + len])
    }

    pub fn write(&mut self, addr: usize, bytes: &[u8]) {
        let mut address = addr;
        for byte in bytes {
            self.memory[address] = *byte;
            address += 1;
        }
    }
}
