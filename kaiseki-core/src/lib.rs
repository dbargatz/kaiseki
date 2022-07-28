use std::sync::{Arc, Mutex};

mod bus;
mod component;
mod cpu;
mod machine;
mod memory;
mod oscillator;

pub use crate::bus::{Bus, BusConnection, BusMessage, BusMessageMetadata};
pub use crate::component::{Component, ComponentId};
pub use crate::cpu::CPU;
pub use crate::machine::Machine;
pub use crate::memory::{MemoryBus, MemoryBusMessage, SimpleRAM, RAM};
pub use crate::oscillator::{Oscillator, OscillatorBus, OscillatorBusMessage};

#[derive(Debug)]
pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Runner<C>
where
    C: 'static + Send + Component,
{
    inner: Arc<Mutex<C>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl<C: 'static + Send + Component> Runner<C> {
    pub fn new(component: C) -> Self {
        let inner = Arc::new(Mutex::new(component));
        Runner {
            inner,
            handle: None,
        }
    }

    pub fn start(&mut self) {
        let inner = self.inner.clone();
        let handle = std::thread::spawn(move || {
            let mut component = inner.lock().unwrap();
            component.start();
        });
        self.handle = Some(handle);
    }

    pub fn stop(&mut self) {
        let handle = std::mem::take(&mut self.handle);
        match handle {
            None => {}
            Some(joiner) => {
                let _ = joiner.join();
            }
        }
    }
}
