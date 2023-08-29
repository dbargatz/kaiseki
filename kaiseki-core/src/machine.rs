use std::io;

use thiserror::Error;

use crate::{
    component::{AddressableComponentError, ExecutableComponent},
    MessageBusError,
};

#[derive(Debug, Error)]
pub enum MachineError {
    #[error(transparent)]
    Addressable(#[from] AddressableComponentError),
    #[error(transparent)]
    MessageBus(#[from] MessageBusError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("failed to load '{0}' into memory at 0x{1:04X}")]
    FileLoad(String, usize),
}

pub type Result<T> = std::result::Result<T, MachineError>;

pub trait Machine: ExecutableComponent {
    fn get_frame(&self) -> (usize, usize, Vec<u8>);
    fn load(&self, file: &str) -> Result<()>;
}
