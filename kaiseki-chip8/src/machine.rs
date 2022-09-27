use std::fs;

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::{
    AddressableBus, AddressableComponent, Component, ComponentId, DisplayBus, ExecutableComponent,
    Machine, MonochromeDisplay, Oscillator, OscillatorBus, RAM, ROM,
};

use crate::cpu::Chip8CPU;

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    #[allow(dead_code)]
    clock_bus: OscillatorBus,
    #[allow(dead_code)]
    display_bus: DisplayBus,
    #[allow(dead_code)]
    memory_bus: AddressableBus,
    cpu: Chip8CPU,
    display: MonochromeDisplay<2048, 64, 32>,
    #[allow(dead_code)]
    interpreter_rom: ROM<0x200>,
    ram: RAM<0xE00>,
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

impl Machine for Chip8Machine {
    fn load(&self, file: &str) -> Result<()> {
        tracing::info!("loading Chip-8 program");
        let program = fs::read(file)?;
        self.ram.write(0x200, &program)?;
        Ok(())
    }
}

impl Chip8Machine {
    pub fn new() -> Result<Chip8Machine> {
        let clock_bus = OscillatorBus::new("clock bus");
        let display_bus = DisplayBus::new("display bus");
        let memory_bus = AddressableBus::new("memory bus");

        let cpu = Chip8CPU::new(&clock_bus, &display_bus, &memory_bus, 0x200);
        let display = MonochromeDisplay::new(&display_bus, &memory_bus);
        let ram = RAM::new("RAM");
        let osc = Oscillator::new(&clock_bus, 500);

        let interpreter_rom = ROM::new("Interpreter ROM", &[]);

        let (_, _) = clock_bus.connect(osc.id(), cpu.id())?;
        let (_, _) = display_bus.connect(cpu.id(), display.id())?;

        memory_bus.map(0x0000..=0x01FF, interpreter_rom.clone())?;
        memory_bus.map(0x0200..=0x0FFF, ram.clone())?;

        let machine = Chip8Machine {
            id: ComponentId::new("Chip-8 Machine"),
            clock_bus,
            display_bus,
            memory_bus,
            cpu,
            display,
            interpreter_rom,
            ram,
            system_clock: osc,
        };
        Ok(machine)
    }
}
