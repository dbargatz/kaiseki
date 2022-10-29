use std::sync::{Arc, Mutex};

use anyhow::Result;

use kaiseki_core::{AddressableBus, AddressableComponent, Component, ComponentId};

#[derive(Clone, Debug)]
pub struct MonochromeDisplayState<const N: usize> {
    #[allow(dead_code)]
    memory_bus: AddressableBus,
    #[allow(dead_code)]
    width: usize,
    #[allow(dead_code)]
    height: usize,
    pixels: [u8; N],
}

#[derive(Clone, Debug)]
pub struct MonochromeDisplay<const N: usize> {
    id: ComponentId,
    state: Arc<Mutex<MonochromeDisplayState<N>>>,
}

impl<const N: usize> Component for MonochromeDisplay<N> {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl<const N: usize> AddressableComponent for MonochromeDisplay<N> {
    fn read(&self, address: usize, length: usize) -> Result<Vec<u8>> {
        tracing::trace!("reading {} bytes from 0x{:08X}", length, address);
        let state = self.state.lock().unwrap();
        let slice = &state.pixels[address..address + length];
        Ok(Vec::from(slice))
    }

    fn write(&self, address: usize, data: &[u8]) -> Result<()> {
        tracing::trace!(
            "writing 0x{:X} bytes to 0x{:04X} - 0x{:04X}",
            data.len(),
            address,
            address + data.len()
        );
        let mut state = self.state.lock().unwrap();
        state.pixels[address..address + data.len()].clone_from_slice(data);
        Ok(())
    }
}

impl<const N: usize> MonochromeDisplay<N> {
    pub fn new(memory_bus: &AddressableBus, width: usize, height: usize) -> Self {
        if width * height != N {
            panic!(
                "width * height of MonochromeDisplay<{}> must equal {}",
                N, N
            );
        }

        Self {
            id: ComponentId::new("Monochrome Display"),
            state: Arc::new(Mutex::new(MonochromeDisplayState {
                memory_bus: memory_bus.clone(),
                width,
                height,
                pixels: [0; N],
            })),
        }
    }
}
