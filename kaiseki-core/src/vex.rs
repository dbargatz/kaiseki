use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::machine::Machine;

pub struct Vex {
    command: String,
    machine: Arc<Mutex<dyn Machine>>,
}

impl Vex {
    pub fn create(machine: impl Machine, command: &str) -> Self {
        Self {
            command: String::from(command),
            machine: Arc::new(Mutex::new(machine)),
        }
    }

    pub async fn destroy(&self) {}

    pub async fn revert(&self) {}
    pub async fn snapshot(&self) {}
    pub async fn start(&self) -> Result<()> {
        let mut machine = self.machine.lock().await;
        machine.load(self.command.as_str())?;
        machine.start().await;
        Ok(())
    }

    pub async fn stop(&self) {}
}
