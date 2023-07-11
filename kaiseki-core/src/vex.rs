use std::sync::Arc;

use anyhow::Result;

use crate::machine::Machine;

#[derive(Clone)]
pub struct Vex {
    command: String,
    machine: Arc<dyn Machine>,
}

impl Vex {
    pub fn create(machine: impl Machine, command: &str) -> Self {
        Self {
            command: String::from(command),
            machine: Arc::new(machine),
        }
    }

    pub async fn destroy(&self) {}

    pub fn get_frame(&self) -> Vec<u8> {
        self.machine.get_frame()
    }

    pub async fn revert(&self) {}
    pub async fn snapshot(&self) {}
    pub async fn start(&self) -> Result<()> {
        let machine = &self.machine;
        machine.load(self.command.as_str())?;
        machine.start().await;
        Ok(())
    }

    pub async fn stop(&self) {}
}
