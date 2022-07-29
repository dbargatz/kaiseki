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
pub use crate::memory::{MemoryBus, MemoryBusMessage, RAM};
pub use crate::oscillator::{Oscillator, OscillatorBus, OscillatorBusMessage};

#[derive(Debug)]
pub struct Error;
pub type Result<T> = std::result::Result<T, Error>;
