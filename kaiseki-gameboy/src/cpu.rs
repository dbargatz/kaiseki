use async_trait::async_trait;
use bytes::{Buf, Bytes};

use kaiseki_core::{
    Component, ComponentId, MemoryBus, MemoryBusMessage, OscillatorBus, OscillatorBusMessage,
};

#[derive(Clone, Debug)]
pub enum SM83Error {
    LoadError,
}
pub type Result<T> = std::result::Result<T, SM83Error>;

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
            let cycle_msg = self.clock_bus.recv_direct(&self.id).await.unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = cycle_msg
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
                for current_cycle in start_cycle..end_cycle {
                    self.execute_cycle(current_cycle).await.unwrap();
                }

                let cycle_end = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycle_budget,
                };
                self.clock_bus
                    .send_direct(&self.id, cycle_end)
                    .await
                    .unwrap();
            }
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

    async fn load(&mut self, address: usize, length: usize) -> Result<Bytes> {
        let msg = MemoryBusMessage::ReadAddress { address, length };

        self.memory_bus.send_direct(&self.id, msg).await.unwrap();
        let response = self.memory_bus.recv_direct(&self.id).await.unwrap();

        if let MemoryBusMessage::ReadResponse { data } = response {
            Ok(data)
        } else {
            tracing::warn!("unexpected message on memory bus: {:?}", response);
            Err(SM83Error::LoadError)
        }
    }

    async fn execute_cycle(&mut self, cycle_number: usize) -> Result<()> {
        let offset = (2 * cycle_number) % 0x800;
        let address = 0x200 + offset;
        let instruction_bytes = self.load(address, 2).await.unwrap().get_u16();
        tracing::debug!(
            "cycle {} | load 0x{:04X} => 0x{:04X}",
            cycle_number,
            address,
            instruction_bytes,
        );
        Ok(())
    }
}
