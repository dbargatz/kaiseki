use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::Bytes;

use crate::component::{AddressableComponent, Component, ComponentId};

#[derive(Clone, Debug)]
struct RAMState<const N: usize> {
    buffer: [u8; N],
    bytes_read: usize,
    bytes_written: usize,
    num_reads: usize,
    num_writes: usize,
}

#[derive(Clone, Debug)]
pub struct RAM<const N: usize> {
    id: ComponentId,
    state: Arc<Mutex<RAMState<N>>>,
}

impl<const N: usize> Component for RAM<N> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl<const N: usize> AddressableComponent for RAM<N> {
    fn read(&self, address: usize, length: usize) -> Result<Bytes> {
        let mut state = self.state.lock().unwrap();
        state.bytes_read += length;
        state.num_reads += 1;
        let slice = &state.buffer[address..address + length];
        Ok(Bytes::copy_from_slice(slice))
    }

    fn write(&self, address: usize, data: &[u8]) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.bytes_written += data.len();
        state.num_writes += 1;
        state.buffer[address..address + data.len()].clone_from_slice(data);
        Ok(())
    }
}

impl<const N: usize> RAM<N> {
    pub fn new(name: &str) -> Self {
        Self {
            id: ComponentId::new(name),
            state: Arc::new(Mutex::new(RAMState {
                buffer: [0; N],
                bytes_read: 0,
                bytes_written: 0,
                num_reads: 0,
                num_writes: 0,
            })),
        }
    }
}
