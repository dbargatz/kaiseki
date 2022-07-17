use async_trait::async_trait;
use component::Component;
use memory::RAM;

mod bus;
mod component;
mod cpu;
mod oscillator;
mod memory;

use crate::bus::SystemBus;
use crate::cpu::CPU;
use crate::oscillator::{Oscillator, OscillatorClient};
use crate::memory::SimpleRAM;

#[derive(Debug)]
pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

#[async_trait(?Send)]
pub trait Machine {
    async fn start(&mut self) -> Result<()>;
}

#[derive(Debug, Default)]
#[allow(non_snake_case)]
#[allow(unused)]
pub struct Chip8Registers {
    V0: u8,
    V1: u8,
    V2: u8,
    V3: u8,
    V4: u8,
    V5: u8,
    V6: u8,
    V7: u8,
    V8: u8,
    V9: u8,
    VA: u8,
    VB: u8,
    VC: u8,
    VD: u8,
    VE: u8,

    // Flags register - do not use as general-purpose register.
    VF: u8,
    VI: u16,
    PC: u16,
    SP: u8,

    /// Delay timer register.
    DT: u8,

    /// Sound timer register.
    ST: u8,
}

impl Chip8Registers {
    pub fn new() -> Self {
        Chip8Registers {
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Chip8Stack {
    pub slots: [u16; 16],
}

impl Chip8Stack {
    pub fn new() -> Self {
        Chip8Stack { slots: [0; 16] }
    }
}

pub type Chip8RAM = SimpleRAM<4096>;

#[derive(Debug)]
pub struct Chip8CPU {
    regs: Chip8Registers,
    stack: Chip8Stack,
}

impl Component for Chip8CPU { }
impl CPU for Chip8CPU { }

impl Chip8CPU {
    pub fn new() -> Self {
        Chip8CPU {
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        }
    }
}

impl OscillatorClient for Chip8CPU {
    fn tick(&mut self) {
        // let pc = self.regs.PC;
        // let value = self.bus.read_u16(pc as usize).
        // println!("PC: 0x{:X} Opcode: 0x{:X}", self.regs.PC, value);
        // self.regs.PC += 2;
    }
}

#[derive(Debug)]
pub struct Chip8Machine<'a> {
    bus: SystemBus,
    cpu: Chip8CPU,
    ram: Chip8RAM,
    system_clock: Oscillator<'a>,
}

#[async_trait(?Send)]
impl<'a> Machine for Chip8Machine<'a> {
    async fn start(&mut self) -> Result<()> {
        println!("starting Chip-8 machine");
        self.cpu.regs.PC = 0x200;
        self.system_clock.start().await.unwrap();
        Ok(())
    }
}

impl<'a> Chip8Machine<'a> {
    pub fn new(program: &[u8]) -> Result<Chip8Machine> {
        let bus = SystemBus::new();
        let cpu = Chip8CPU::new();
        let ram = Chip8RAM::new();
        let osc = Oscillator::new(500);

        bus.connect(&ram);
        bus.connect(&cpu);
        bus.connect(&osc);

        ram.write(0x200, program);

        let machine = Chip8Machine { bus, cpu, ram, system_clock: osc };
        Ok(machine)

        // TODO: BUS BETWEEN CPU AND MEMORY
        // TODO: BUS BETWEEN OSCILLATOR AND CPU (REMOVE OSCILLATOR CLIENT)
        // TODO: MAKE EVERYTHING A COMPONENT
    }
}
