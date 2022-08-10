use std::sync::{Arc, RwLock};

use anyhow::Result;
use bytes::Bytes;

use crate::bus::AddressableBus;
use crate::component::{AddressableComponent, Component, ComponentId};

pub type MemoryBus = AddressableBus;

#[derive(Clone, Debug)]
pub struct RAM<const N: usize> {
    id: ComponentId,
    memory: Arc<RwLock<[u8; N]>>,
}

impl<const N: usize> Component for RAM<N> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl<const N: usize> AddressableComponent for RAM<N> {
    fn read(&self, address: usize, length: usize) -> Result<Bytes> {
        let memory = self.memory.read().unwrap();
        let slice = &memory[address..address + length];
        Ok(Bytes::copy_from_slice(slice))
    }

    fn write(&self, address: usize, data: &[u8]) -> Result<()> {
        let mut addr = address;
        let mut memory = self.memory.write().unwrap();
        for byte in data {
            memory[addr] = *byte;
            addr += 1;
        }
        Ok(())
    }
}

impl<const N: usize> RAM<N> {
    pub fn new() -> Self {
        let id = ComponentId::new("RAM");
        RAM {
            id,
            memory: Arc::new(RwLock::new([0; N])),
        }
    }
}
