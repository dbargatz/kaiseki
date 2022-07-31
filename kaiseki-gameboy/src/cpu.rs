use async_trait::async_trait;
use bytes::{Buf, Bytes};

use kaiseki_core::{
    BusConnection, Component, ComponentId, MemoryBus, MemoryBusMessage, OscillatorBus,
    OscillatorBusMessage,
};

#[derive(Clone, Debug)]
pub enum SM83Error {
    LoadError,
}
pub type Result<T> = std::result::Result<T, SM83Error>;

#[derive(Debug)]
pub struct SM83Cpu {
    id: ComponentId,
    clock_bus: BusConnection<OscillatorBusMessage>,
    memory_bus: BusConnection<MemoryBusMessage>,
}

#[async_trait]
impl Component for SM83Cpu {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        loop {
            let cycle_msg = self.clock_bus.recv().await.unwrap();
            if let OscillatorBusMessage::CycleBatchStart {
                start_cycle,
                cycle_budget,
            } = cycle_msg
            {
                let end_cycle = start_cycle + cycle_budget;
                tracing::info!("executing cycles {} - {}", start_cycle, end_cycle);
                self.execute_cycles(start_cycle, end_cycle).await.unwrap();

                let cycle_end = OscillatorBusMessage::CycleBatchEnd {
                    start_cycle,
                    cycles_spent: cycle_budget,
                };
                self.clock_bus.send(cycle_end).await.unwrap();
            }
        }
    }
}

impl SM83Cpu {
    pub fn new(clock_bus: &mut OscillatorBus, memory_bus: &mut MemoryBus) -> Self {
        let id = ComponentId::new_v4();
        let clock_conn = clock_bus.connect(&id);
        let mem_conn = memory_bus.connect(&id);
        SM83Cpu {
            id,
            clock_bus: clock_conn,
            memory_bus: mem_conn,
        }
    }

    async fn load(&mut self, address: usize, length: usize) -> Result<Bytes> {
        let msg = MemoryBusMessage::ReadAddress { address, length };

        self.memory_bus.send(msg).await.unwrap();
        let response = self.memory_bus.recv().await.unwrap();

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

    pub async fn execute_cycles(&mut self, start_cycle: usize, end_cycle: usize) -> Result<()> {
        let start = std::time::Instant::now();
        for current_cycle in start_cycle..end_cycle {
            self.execute_cycle(current_cycle).await.unwrap();
        }
        tracing::info!(
            "elapsed executing cycles {} - {}: {:.3}s",
            start_cycle,
            end_cycle,
            start.elapsed().as_secs_f64()
        );
        Ok(())
    }
}
