use kaiseki_core::{BusConnection, BusMessage, Component, SimpleRAM, CPU};

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

impl Default for Chip8Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8Stack {
    pub fn new() -> Self {
        Chip8Stack { slots: [0; 16] }
    }
}

pub type Chip8RAM = SimpleRAM<4096>;

#[derive(Debug)]
pub struct Chip8CPU {
    bus: Option<BusConnection>,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

impl Component for Chip8CPU {
    fn connect_to_bus(&mut self, bus: BusConnection) {
        self.bus = Some(bus);
    }

    fn start(&mut self) {
        let bus = self.bus.as_mut().unwrap();
        loop {
            let msg = bus.recv().unwrap();
            if let BusMessage::OscillatorTick { cycle } = msg {
                println!("CPU received tick {} (stack: {:#?}", cycle, self.stack);
            }
        }
    }
}

impl CPU for Chip8CPU {}

impl Chip8CPU {
    pub fn new(initial_pc: u16) -> Self {
        let mut cpu = Chip8CPU {
            bus: None,
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }
}
