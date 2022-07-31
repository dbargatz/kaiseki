use async_trait::async_trait;
use bytes::{Buf, Bytes};

use kaiseki_core::{
    BusConnection, Component, ComponentId, CpuComponent, CpuResult, MemoryBus, MemoryBusMessage,
};

#[derive(Clone, Debug)]
pub enum SM83Error {
    LoadError,
}
pub type Result<T> = std::result::Result<T, SM83Error>;

#[derive(Debug)]
pub struct SM83Cpu {
    id: ComponentId,
    memory_bus: BusConnection<MemoryBusMessage>,
}

#[async_trait]
impl Component for SM83Cpu {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {}
}

impl CpuComponent for SM83Cpu {
    fn execute_cycles(&mut self, start_cycle: usize, end_cycle: usize) -> CpuResult<()> {
        for current_cycle in start_cycle..end_cycle {
            self.execute_cycle(current_cycle).unwrap();
        }
        Ok(())
    }
}

impl SM83Cpu {
    pub fn new(memory_bus: &mut MemoryBus) -> Self {
        let id = ComponentId::new_v4();
        let mem_conn = memory_bus.connect(&id);
        let cpu = SM83Cpu {
            id,
            memory_bus: mem_conn,
        };
        cpu
    }

    fn load(&mut self, address: usize, length: usize) -> Result<Bytes> {
        let msg = MemoryBusMessage::ReadAddress { address, length };

        self.memory_bus.blocking_send(msg).unwrap();
        let response = self.memory_bus.blocking_recv().unwrap();

        if let MemoryBusMessage::ReadResponse { data } = response {
            Ok(data)
        } else {
            tracing::warn!("unexpected message on memory bus: {:?}", response);
            Err(SM83Error::LoadError)
        }
    }

    fn execute_cycle(&mut self, cycle_number: usize) -> Result<()> {
        let offset = (2 * cycle_number) % 0x800;
        let address = 0x200 + offset;
        let instruction_bytes = self.load(address, 2).unwrap().get_u16();
        tracing::debug!(
            "cycle {} | load 0x{:04X} => 0x{:04X}",
            cycle_number,
            address,
            instruction_bytes,
        );
        Ok(())
    }
}
