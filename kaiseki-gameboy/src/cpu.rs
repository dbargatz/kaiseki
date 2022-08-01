use anyhow::Result;
use async_trait::async_trait;
use bytes::Buf;

use kaiseki_core::{Component, ComponentId, MemoryBus, OscillatorBus};

#[derive(Debug)]
pub struct SM83Cpu {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
}

#[async_trait]
impl Component for SM83Cpu {
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

impl SM83Cpu {
    pub fn new(clock_bus: &OscillatorBus, memory_bus: &MemoryBus) -> Self {
        let id = ComponentId::new_v4();
        SM83Cpu {
            id,
            clock_bus: clock_bus.clone(),
            memory_bus: memory_bus.clone(),
        }
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
            "cycle {} | load 0x{:04X} => 0x{:04X}",
            cycle_number,
            address,
            instruction_bytes,
        );
        Ok(())
    }
}
