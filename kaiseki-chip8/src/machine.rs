use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::{
    Component, ComponentId, Cpu, Machine, MemoryBus, Oscillator, OscillatorBus, Result, RAM,
};

use crate::cpu::{Chip8CPU, Chip8RAM};

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
    cpu: Cpu<Chip8CPU>,
    ram: Chip8RAM,
    system_clock: Oscillator,
}

#[async_trait]
impl Component for Chip8Machine {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        tracing::info!("starting Chip-8 machine");

        let mut futures = FuturesUnordered::new();
        futures.push(self.clock_bus.start());
        futures.push(self.memory_bus.start());

        futures.push(self.cpu.start());
        futures.push(self.ram.start());
        futures.push(self.system_clock.start());

        while futures.next().await.is_some() {
            tracing::info!("component task finished");
        }
    }
}

impl Machine for Chip8Machine {}

impl Chip8Machine {
    pub fn new(program: &[u8]) -> Result<Chip8Machine> {
        let mut clock_bus = OscillatorBus::new();
        let mut memory_bus = MemoryBus::new();

        let cpu = Cpu::new(&mut clock_bus, Chip8CPU::new(&mut memory_bus, 0x200));
        let mut ram = Chip8RAM::new(&mut memory_bus);
        let osc = Oscillator::new(&mut clock_bus, 500);

        ram.write(0x200, program);

        let machine = Chip8Machine {
            id: ComponentId::new_v4(),
            clock_bus,
            memory_bus,
            cpu,
            ram,
            system_clock: osc,
        };
        Ok(machine)
    }
}
