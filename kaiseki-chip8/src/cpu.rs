use std::fmt;

use async_trait::async_trait;
use bytes::{Bytes, Buf};

use kaiseki_core::{
    BusConnection, Component, ComponentId, CpuComponent, CpuResult, MemoryBus, MemoryBusMessage,
    SimpleRAM,
};

#[derive(Clone, Debug)]
pub enum Chip8CpuError {
    LoadError,
}
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
    memory_bus: BusConnection<MemoryBusMessage>,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

#[async_trait]
impl Component for Chip8CPU {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {}
}

impl CpuComponent for Chip8CPU {
    fn execute_cycles(
        &mut self,
        start_cycle: usize,
        end_cycle: usize,
    ) -> CpuResult<()> {
        for current_cycle in start_cycle..end_cycle {
            self.execute_cycle(current_cycle).unwrap();
        }
        Ok(())
    }
}

impl Chip8CPU {
    pub fn new(memory_bus: &mut MemoryBus, initial_pc: u16) -> Self {
        let id = ComponentId::new_v4();
        let mem_conn = memory_bus.connect(&id);
        let mut cpu = Chip8CPU {
            id,
            memory_bus: mem_conn,
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }

    fn load(&mut self, address: usize, length: usize) -> Result<Bytes> {
        let msg = MemoryBusMessage::ReadAddress {
            address,
            length,
        };

        self.memory_bus.blocking_send(msg).unwrap();
        let response = self.memory_bus.blocking_recv().unwrap();

        if let MemoryBusMessage::ReadResponse { data } = response {
            Ok(data)
        } else {
            tracing::warn!("unexpected message on memory bus: {:?}", response);
            Err(Chip8CpuError::LoadError)
        }
    }

    fn execute_cycle(&mut self, cycle_number: usize) -> Result<()> {
        let offset = (2 * cycle_number) % 0x800;
        let address = 0x200 + offset;
        let instruction_bytes = self.load(address, 2).unwrap().get_u16();
        tracing::debug!(
            "cycle {} | load 0x{:04X} => 0x{:04X} | stack: {:?}",
            cycle_number,
            address,
            instruction_bytes,
            self.stack
        );
        Ok(())
    }
}
