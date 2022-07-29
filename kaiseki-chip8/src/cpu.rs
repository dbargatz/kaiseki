use kaiseki_core::{
    BusConnection, Component, ComponentId, MemoryBus, MemoryBusMessage, OscillatorBus,
    OscillatorBusMessage, SimpleRAM, CPU,
};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Chip8CpuError;

pub type Result<T> = std::result::Result<T, Chip8CpuError>;

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

pub struct Chip8Stack {
    pub slots: [u16; 16],
}

impl fmt::Debug for Chip8Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip-8 Stack").finish()
    }
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
    id: ComponentId,
    clock_bus: BusConnection<OscillatorBusMessage>,
    memory_bus: BusConnection<MemoryBusMessage>,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

impl Component for Chip8CPU {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&self) {
        loop {
            let cycle_msg = self.clock_bus.recv().unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = cycle_msg
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);

                let mut cycles_executed: usize = 0;
                for current_cycle in start_cycle..end_cycle {
                    match self.execute_cycle(current_cycle) {
                        Ok(_) => cycles_executed += 1,
                        Err(_) => break,
                    }
                }

                let cycle_end = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycles_executed,
                };
                self.clock_bus.send(cycle_end);
            }
        }
    }
}

impl CPU for Chip8CPU {}

impl Chip8CPU {
    pub fn new(clock_bus: &mut OscillatorBus, memory_bus: &mut MemoryBus, initial_pc: u16) -> Self {
        let id = ComponentId::new_v4();
        let clock_conn = clock_bus.connect(&id);
        let mem_conn = memory_bus.connect(&id);
        let mut cpu = Chip8CPU {
            id,
            clock_bus: clock_conn,
            memory_bus: mem_conn,
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }

    fn execute_cycle(&self, cycle_number: usize) -> Result<()> {
        let offset = (2 * cycle_number) % 0x800;
        let address = 0x200 + offset;
        let msg = MemoryBusMessage::ReadAddress {
            address,
            length: 0x2,
        };
        self.memory_bus.send(msg);
        let response = self.memory_bus.recv().unwrap();
        if let MemoryBusMessage::ReadResponse { data } = response {
            tracing::trace!(
                "cycle {} | simulated read instruction | 0x{:04X}: 0x{:04X} | stack: {:?}",
                cycle_number,
                address,
                data,
                self.stack
            );
        } else {
            tracing::warn!("unexpected message on memory bus: {:?}", response);
        }
        Ok(())
    }
}
