use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::{Component, ComponentId, Machine, MemoryBus, Oscillator, OscillatorBus, RAM};

use crate::cpu::Chip8CPU;

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
    cpu: Chip8CPU,
    ram: RAM<4096>,
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
    pub async fn new(program: &[u8]) -> Result<Chip8Machine> {
        let clock_bus = OscillatorBus::new();
        let memory_bus = MemoryBus::new();

        let cpu = Chip8CPU::new(&clock_bus, &memory_bus, 0x200);
        let mut ram = RAM::new(&memory_bus);
        let osc = Oscillator::new(&clock_bus, 500);

        clock_bus.connect(&cpu.id()).await.unwrap();
        clock_bus.connect(&osc.id()).await.unwrap();

        memory_bus.connect(&cpu.id()).await.unwrap();
        memory_bus.connect(&ram.id()).await.unwrap();

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
