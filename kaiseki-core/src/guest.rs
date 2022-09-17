use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::machine::Machine;

pub struct Guest {
    command: String,
    machine: Arc<Mutex<dyn Machine>>,
}

impl Guest {
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
        let mut machine = self.machine.lock().unwrap();
        machine.load(self.command.as_str())?;
        machine.start().await;
        Ok(())
    }

    pub async fn stop(&self) {}
}
