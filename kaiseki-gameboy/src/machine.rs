use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::{Component, ComponentId, Machine, MemoryBus, Oscillator, OscillatorBus, RAM};

use crate::cpu::SM83Cpu;

#[derive(Debug)]
pub struct GameboyMachine {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
    cpu: SM83Cpu,
    ram: RAM<8192>,
    system_clock: Oscillator,
}

#[async_trait]
impl Component for GameboyMachine {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        tracing::info!("starting gameboy machine");

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

impl Machine for GameboyMachine {}

impl GameboyMachine {
    pub async fn new() -> Result<GameboyMachine> {
        let clock_bus = OscillatorBus::new();
        let memory_bus = MemoryBus::new();

        let cpu = SM83Cpu::new(&clock_bus, &memory_bus);
        let ram = RAM::new(&memory_bus);
        let osc = Oscillator::new(&clock_bus, 4_000_000);

        clock_bus.connect(&cpu.id()).await.unwrap();
        clock_bus.connect(&osc.id()).await.unwrap();

        memory_bus.connect(&cpu.id()).await.unwrap();
        memory_bus.connect(&ram.id()).await.unwrap();

        let machine = GameboyMachine {
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
