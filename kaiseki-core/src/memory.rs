use async_trait::async_trait;
use bytes::Bytes;

use crate::bus::{Bus, BusMessage};
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

#[derive(Debug)]
pub struct RAM<const N: usize> {
    id: ComponentId,
    bus: MemoryBus,
    memory: [u8; N],
}

#[async_trait]
impl<const N: usize> Component for RAM<N> {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        loop {
            if let Ok(MemoryBusMessage::ReadAddress { address, length }) =
                self.bus.recv_direct(&self.id).await
            {
                tracing::trace!("read request: {} bytes at 0x{:X}", length, address);
                let mem = Bytes::copy_from_slice(&self.memory[address..address + length]);
                let response = MemoryBusMessage::ReadResponse { data: mem };
                self.bus.send_direct(&self.id, response).await.unwrap();
            }
        }
    }
}

impl<const N: usize> RAM<N> {
    pub fn new(memory_bus: &MemoryBus) -> Self {
        let id = ComponentId::new_v4();
        RAM {
            id,
            bus: memory_bus.clone(),
            memory: [0; N],
        }
    }

    pub fn read(&self, addr: usize, len: usize) -> Bytes {
        Bytes::copy_from_slice(&self.memory[addr..addr + len])
    }

    // fn read_u16(&self, addr: usize) -> u16 {
    //     let slice = self.read(addr, 2);
    //     let value: u16 = (slice[0] as u16) << 8 | slice[1] as u16;
    //     value
    // }

    pub fn write(&mut self, addr: usize, bytes: &[u8]) {
        let mut address = addr;
        for byte in bytes {
            self.memory[address] = *byte;
            address += 1;
        }
    }
}
