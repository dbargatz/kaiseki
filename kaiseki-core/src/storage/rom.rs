use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};

use crate::component::{AddressableComponent, Component, ComponentId};

#[derive(Clone, Debug)]
struct ROMState<const N: usize> {
    buffer: [u8; N],
    bytes_read: usize,
    num_reads: usize,
}

#[derive(Clone, Debug)]
pub struct ROM<const N: usize> {
    id: ComponentId,
    state: Arc<Mutex<ROMState<N>>>,
}

impl<const N: usize> Component for ROM<N> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl<const N: usize> AddressableComponent for ROM<N> {
    fn read(&self, address: usize, length: usize) -> Result<Vec<u8>> {
        let mut state = self.state.lock().unwrap();
        state.bytes_read += length;
        state.num_reads += 1;
        let slice = &state.buffer[address..address + length];
        Ok(Vec::from(slice))
    }

    fn write(&self, _: usize, _: &[u8]) -> Result<()> {
        Err(anyhow!("cannot write to ROM"))
    }
}

impl<const N: usize> ROM<N> {
    pub fn new(name: &str, contents: &[u8]) -> Self {
        let mut buffer = [0; N];
        buffer[..contents.len()].copy_from_slice(contents);

        Self {
            id: ComponentId::new(name),
            state: Arc::new(Mutex::new(ROMState {
                buffer,
                bytes_read: 0,
                num_reads: 0,
            })),
        }
    }
}
