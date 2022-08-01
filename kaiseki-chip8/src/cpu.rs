use async_trait::async_trait;
use bytes::Buf;

use kaiseki_core::{Component, ComponentId, MemoryBus, OscillatorBus};

use super::registers::Chip8Registers;
use super::stack::Chip8Stack;

#[derive(Clone, Debug)]
pub enum Chip8CpuError {
    LoadError,
}
pub type Result<T> = std::result::Result<T, Chip8CpuError>;

#[derive(Debug)]
pub struct Chip8CPU {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
    regs: Chip8Registers,
    stack: Chip8Stack,
}

#[async_trait]
impl Component for Chip8CPU {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        loop {
            let (start_cycle, cycle_budget) = self.clock_bus.wait(&self.id).await.unwrap();
            let end_cycle = start_cycle + cycle_budget;
            tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
            for current_cycle in start_cycle..end_cycle {
                self.execute_cycle(current_cycle).await.unwrap();
            }

            self.clock_bus
                .complete(&self.id, start_cycle, cycle_budget)
                .await
                .unwrap();
        }
    }
}

impl Chip8CPU {
    pub fn new(clock_bus: &OscillatorBus, memory_bus: &MemoryBus, initial_pc: u16) -> Self {
        let id = ComponentId::new_v4();
        let mut cpu = Chip8CPU {
            id,
            clock_bus: clock_bus.clone(),
            memory_bus: memory_bus.clone(),
            regs: Chip8Registers::new(),
            stack: Chip8Stack::new(),
        };
        cpu.regs.PC = initial_pc;
        cpu
    }

    async fn execute_cycle(&mut self, cycle_number: usize) -> Result<()> {
        let offset = (2 * cycle_number) % 0x800;
        let address = 0x200 + offset;
        let instruction_bytes = self
            .memory_bus
            .read(&self.id, address, 2)
            .await
            .unwrap()
            .get_u16();
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
