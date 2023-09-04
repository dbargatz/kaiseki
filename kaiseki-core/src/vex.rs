use std::sync::Arc;

use thiserror::Error;

use crate::machine::{Machine, MachineError};

#[derive(Debug, Error, PartialEq)]
pub enum VexError {
    #[error(transparent)]
    Load(#[from] MachineError),
}

pub type Result<T> = std::result::Result<T, VexError>;

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

    pub fn get_frame(&self) -> (usize, usize, Vec<u8>) {
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
