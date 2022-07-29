mod bus;
mod component;
mod cpu;
mod machine;
mod memory;
mod oscillator;

pub use crate::bus::{Bus, BusConnection, BusMessage};
pub use crate::component::{Component, ComponentId};
pub use crate::cpu::{Cpu, CpuComponent, CpuResult};
pub use crate::machine::Machine;
pub use crate::memory::{MemoryBus, MemoryBusMessage, SimpleRAM, RAM};
pub use crate::oscillator::{Oscillator, OscillatorBus, OscillatorBusMessage};

#[derive(Debug)]
pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;

// #[derive(Debug)]
// pub struct Runner<C>
// where
//     C: 'static + Send + Component,
// {
//     inner: Arc<Mutex<C>>,
//     handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
// }

// impl<C: 'static + Send + Component> Runner<C> {
//     pub fn new(component: C) -> Self {
//         let inner = Arc::new(Mutex::new(component));
//         let handle = Arc::new(Mutex::new(None));
//         Runner { inner, handle }
//     }

//     pub fn start(&mut self) {
//         let inner = self.inner.clone();
//         let handle = std::thread::spawn(move || {
//             let component = inner.lock().unwrap();
//             component.start();
//         });

//         let mut guard = self.handle.lock().unwrap();
//         *guard = Some(handle);
//     }

//     pub fn stop(&self) {
//         let guard = self.handle.lock().unwrap().take();
//         match guard {
//             None => {}
//             Some(joiner) => {
//                 let _ = joiner.join();
//             }
//         }
//     }
// }
