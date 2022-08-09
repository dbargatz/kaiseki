use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::{
    AddressableComponent, Component, ComponentId, DisplayBus, ExecutableComponent, Machine,
    MemoryBus, MonochromeDisplay, Oscillator, OscillatorBus, RAM,
};

use crate::cpu::Chip8CPU;

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    clock_bus: OscillatorBus,
    display_bus: DisplayBus,
    memory_bus: MemoryBus,
    cpu: Chip8CPU,
    display: MonochromeDisplay<2048, 64, 32>,
    ram: RAM<4096>,
    system_clock: Oscillator,
}

impl Component for Chip8Machine {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl ExecutableComponent for Chip8Machine {
    async fn start(&mut self) {
        tracing::info!("starting Chip-8 machine");

        let mut futures = FuturesUnordered::new();

        futures.push(self.cpu.start());
        futures.push(self.display.start());
        futures.push(self.system_clock.start());

        while futures.next().await.is_some() {
            tracing::info!("component task finished");
        }
    }
}

impl Machine for Chip8Machine {}

impl Chip8Machine {
    pub async fn new(program: &[u8]) -> Result<Chip8Machine> {
        let clock_bus = OscillatorBus::new("clock bus");
        let display_bus = DisplayBus::new("display bus");
        let memory_bus = MemoryBus::new("memory bus");

        let cpu = Chip8CPU::new(&clock_bus, &display_bus, &memory_bus, 0x200);
        let display = MonochromeDisplay::new(&display_bus, &memory_bus);
        let ram = RAM::new(&memory_bus);
        let osc = Oscillator::new(&clock_bus, 500);

        let (_, _) = clock_bus.connect(osc.id(), cpu.id()).unwrap();
        let (_, _) = display_bus.connect(cpu.id(), display.id()).unwrap();
        memory_bus.map(ram.clone(), 0x0000, 0x1000, false).unwrap();
        ram.write(0x200, program).unwrap();

        let machine = Chip8Machine {
            id: ComponentId::new("Chip-8 Machine"),
            clock_bus,
            display_bus,
            memory_bus,
            cpu,
            display,
            ram,
            system_clock: osc,
        };
        Ok(machine)
    }
}
