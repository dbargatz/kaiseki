use anyhow::Result;
use async_trait::async_trait;
use bytes::Buf;

use kaiseki_core::{
    Component, ComponentId, ExecutableComponent, MemoryBus, OscillatorBus, OscillatorBusMessage,
};

#[derive(Debug)]
pub struct SM83Cpu {
    id: ComponentId,
    clock_bus: OscillatorBus,
    memory_bus: MemoryBus,
}

impl Component for SM83Cpu {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

#[async_trait]
impl ExecutableComponent for SM83Cpu {
    async fn start(&mut self) {
        loop {
            let (message, responder) = self.clock_bus.recv(&self.id).await.unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = message
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
                for current_cycle in start_cycle..end_cycle {
                    self.execute_cycle(current_cycle).await.unwrap();
                }
                let response = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycle_budget,
                };
                responder.unwrap().send(response).unwrap();
            }
        }
    }
}

impl SM83Cpu {
    pub fn new(clock_bus: &OscillatorBus, memory_bus: &MemoryBus) -> Self {
        let id = ComponentId::new("SM83 CPU");
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
