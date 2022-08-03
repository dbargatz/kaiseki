use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

use crate::bus::{Bus, BusError, BusMessage};
use crate::component::{Component, ComponentId, ExecutableComponent};

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
        self.broadcast(id, request).await.unwrap();

        loop {
            let (from, response) = self.recv(id).await.unwrap();
            if let MemoryBusMessage::ReadResponse { data } = response {
                return Ok(data);
            } else {
                tracing::trace!(
                    "{}: ignoring message {:?} from {} while waiting for ReadResponse",
                    id,
                    response,
                    from
                );
            }
        }
    }

    pub async fn write(
        &self,
        id: &ComponentId,
        address: usize,
        data: Bytes,
    ) -> Result<(), BusError> {
        let request = MemoryBusMessage::WriteAddress { address, data };
        self.broadcast(id, request).await.unwrap();

        loop {
            let (from, response) = self.recv(id).await.unwrap();
            if let MemoryBusMessage::WriteResponse = response {
                return Ok(());
            } else {
                tracing::trace!(
                    "{}: ignoring message {:?} from {} while waiting for WriteResponse",
                    id,
                    response,
                    from
                );
            }
        }
    }
}

#[derive(Debug)]
pub struct RAM<const N: usize> {
    id: ComponentId,
    bus: MemoryBus,
    memory: [u8; N],
}

impl<const N: usize> Component for RAM<N> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl<const N: usize> ExecutableComponent for RAM<N> {
    async fn start(&mut self) {
        loop {
            if let Ok((from, MemoryBusMessage::ReadAddress { address, length })) =
                self.bus.recv(&self.id).await
            {
                tracing::trace!("read request: {} bytes at 0x{:X}", length, address);
                let mem = Bytes::copy_from_slice(&self.memory[address..address + length]);
                let response = MemoryBusMessage::ReadResponse { data: mem };
                self.bus.send(&self.id, &from, response).await.unwrap();
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
