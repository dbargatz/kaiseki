use async_trait::async_trait;
use bytes::Bytes;

use crate::bus::{Bus, BusConnection, BusMessage};
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

pub trait RAM: Component {
    fn read(&self, addr: usize, len: usize) -> Bytes;
    fn read_u8(&self, addr: usize) -> u8;
    fn read_u16(&self, addr: usize) -> u16;
    //fn read_u32(&self, addr: usize) -> u32;
    //fn read_u64(&self, addr: usize) -> u64;

    fn write(&mut self, addr: usize, bytes: &[u8]);
}

#[derive(Debug)]
pub struct SimpleRAM<const N: usize> {
    id: ComponentId,
    bus: BusConnection<MemoryBusMessage>,
    memory: [u8; N],
}

#[async_trait]
impl<const N: usize> Component for SimpleRAM<N> {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        loop {
            if let Ok(MemoryBusMessage::ReadAddress { address, length }) = self.bus.recv().await {
                tracing::trace!("read request: {} bytes at 0x{:X}", length, address);
                let end_addr = address + length;
                let slice: &[u8] = &self.memory[address..end_addr];
                let mem = bytes::Bytes::copy_from_slice(slice);
                let response = MemoryBusMessage::ReadResponse { data: mem };
                self.bus.send(response).await.unwrap();
            }
        }
    }
}

impl<const N: usize> RAM for SimpleRAM<N> {
    fn read(&self, addr: usize, len: usize) -> Bytes {
        Bytes::copy_from_slice(&self.memory[addr..addr + len])
    }

    fn read_u8(&self, addr: usize) -> u8 {
        let slice = self.read(addr, 1);
        slice[0]
    }

    fn read_u16(&self, addr: usize) -> u16 {
        let slice = self.read(addr, 2);
        let value: u16 = (slice[0] as u16) << 8 | slice[1] as u16;
        value
    }

    fn write(&mut self, addr: usize, bytes: &[u8]) {
        let mut address = addr;
        for byte in bytes {
            self.memory[address] = *byte;
            address += 1;
        }
    }
}

impl<const N: usize> SimpleRAM<N> {
    pub fn new(bus: &mut Bus<MemoryBusMessage>) -> Self {
        let id = ComponentId::new_v4();
        let conn = bus.connect(&id);
        SimpleRAM {
            id,
            bus: conn,
            memory: [0; N],
        }
    }
}
