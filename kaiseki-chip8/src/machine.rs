use std::fs;

use async_trait::async_trait;
use futures::{stream::FuturesUnordered, StreamExt};

use kaiseki_core::machine::{self, Machine};
use kaiseki_core::{
    AddressableBus, AddressableComponent, Component, ComponentId, ExecutableComponent, Oscillator,
    OscillatorBus, RAM, ROM,
};

use crate::cpu::Chip8CPU;
use crate::display::MonochromeDisplay;

#[derive(Debug)]
pub struct Chip8Machine {
    id: ComponentId,
    #[allow(dead_code)]
    clock_bus: OscillatorBus,
    #[allow(dead_code)]
    memory_bus: AddressableBus,
    cpu: Chip8CPU,
    #[allow(dead_code)]
    display: MonochromeDisplay<2048>,
    #[allow(dead_code)]
    interpreter_rom: ROM<0x200>,
    #[allow(dead_code)]
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
    async fn start(&self) {
        tracing::info!("starting Chip-8 machine");

        let mut futures = FuturesUnordered::new();

        futures.push(self.cpu.start());
        futures.push(self.system_clock.start());

        while futures.next().await.is_some() {
            tracing::info!("component task finished");
        }
    }
}

impl Machine for Chip8Machine {
    fn get_frame(&self) -> (usize, usize, Vec<u8>) {
        let mono_frame = self.memory_bus.read(0x1000, 0x100).unwrap();
        let mut rgb_frame = Vec::new();

        for byte in mono_frame {
            for bit_idx in 0..=7 {
                let pixel = (byte >> (7 - bit_idx)) & 0x01;
                match pixel {
                    0 => {
                        rgb_frame.push(0x00);
                        rgb_frame.push(0x00);
                        rgb_frame.push(0x00);
                    }
                    1 => {
                        rgb_frame.push(0xFF);
                        rgb_frame.push(0xFF);
                        rgb_frame.push(0xFF);
                    }
                    _ => {
                        panic!(
                            "pixel value is {}; should be single bit, must be 0 or 1!",
                            pixel
                        );
                    }
                }
            }
        }

        (64, 32, rgb_frame)
    }

    fn load(&self, file: &str) -> machine::Result<()> {
        tracing::info!("loading Chip-8 program");
        let program = fs::read(file)
            .map_err(|_| machine::MachineError::FileLoad(String::from(file), 0x200))?;
        self.memory_bus.write(0x200, &program)?;
        Ok(())
    }
}

impl Chip8Machine {
    pub fn new() -> machine::Result<Chip8Machine> {
        let clock_bus = OscillatorBus::new("clock bus");
        let memory_bus = AddressableBus::new("memory bus");

        let cpu = Chip8CPU::new(&clock_bus, &memory_bus, 0x200);
        let display = MonochromeDisplay::new(&memory_bus, 64, 32);
        let ram = RAM::new("RAM");
        let osc = Oscillator::new(&clock_bus, 500);

        let interpreter_rom = ROM::new("Interpreter ROM", &[]);

        // osc <----clock_bus----> cpu
        // cpu <---memory_bus----> rom[0x0000 - 0x01FF]
        // cpu <---memory_bus----> ram[0x0200 - 0x0FFF]
        // cpu <---memory_bus----> display[0x1000 - 0x1100]

        let (_, _) = clock_bus.connect(osc.id(), cpu.id())?;

        memory_bus.map(0x0000..=0x01FF, interpreter_rom.clone())?;
        memory_bus.map(0x0200..=0x0FFF, ram.clone())?;
        memory_bus.map(0x1000..=0x1100, display.clone())?;

        let machine = Chip8Machine {
            id: ComponentId::new("Chip-8 Machine"),
            clock_bus,
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
