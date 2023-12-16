use std::fmt;
use std::ops::RangeInclusive;

use async_trait::async_trait;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct ComponentId {
    name: String,
    uuid: Uuid,
}

impl Clone for ComponentId {
    fn clone(&self) -> Self {
        Self {
            name: String::from(self.name.as_str()),
            uuid: self.uuid,
        }
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_str())
    }
}

impl ComponentId {
    pub fn new(name: &str) -> Self {
        ComponentId {
            name: String::from(name),
            uuid: Uuid::new_v4(),
        }
    }
}

pub trait Component: 'static + Send + Sync {
    fn id(&self) -> &ComponentId;
}

impl PartialEq for dyn Component + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for dyn Component + '_ {}

#[derive(Debug, Error, PartialEq)]
pub enum AddressableComponentError {
    #[error("no component is mapped at address 0x{0:04X}")]
    NoComponentMappedAtAddress(usize),
    #[error("component {0} failed to read {2} bytes at address 0x{1:04X}")]
    ComponentReadFailed(ComponentId, usize, usize),
    #[error("component {0} failed to write {2} bytes at address 0x{1:04X}")]
    ComponentWriteFailed(ComponentId, usize, usize),
    #[error("cannot map component {0} to {1:?}; conflicts with already-mapped component {2}")]
    MappingConflict(ComponentId, RangeInclusive<usize>, ComponentId),
}

pub type Result<T> = std::result::Result<T, AddressableComponentError>;

pub trait AddressableComponent: Component {
    fn read(&self, address: usize, length: usize) -> Result<Vec<u8>>;
    fn write(&self, address: usize, data: &[u8]) -> Result<()>;
}

impl PartialEq for dyn AddressableComponent + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for dyn AddressableComponent + '_ {}

#[async_trait]
pub trait ExecutableComponent: Component {
    async fn start(&self);
}
