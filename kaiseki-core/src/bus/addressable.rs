use std::fmt;
use std::ops::Range;
use std::sync::{Arc, RwLock};

use anyhow::Result;
use bytes::Bytes;
use rangemap::RangeMap;
use thiserror::Error;

use crate::component::{AddressableComponent, Component, ComponentId};

#[derive(Debug, Error, PartialEq)]
pub enum AddressableBusError {
    #[error("address 0x{0:04X} is invalid")]
    InvalidAddress(usize),
    #[error("no component is mapped at address 0x{0:04X}")]
    NoComponentMappedAtAddress(usize),
    #[error("component {0} is already mapped at address 0x{1:04X}")]
    ComponentAlreadyMappedAtAddress(ComponentId, usize),
}

struct AddressableBusState {
    mappings: RangeMap<usize, Arc<dyn AddressableComponent>>,
}

impl AddressableBusState {
    pub fn new() -> Self {
        Self {
            mappings: RangeMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct AddressableBus {
    id: ComponentId,
    state: Arc<RwLock<AddressableBusState>>,
}

impl fmt::Debug for AddressableBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AddressableBus[{}]", self.id)
    }
}

impl Component for AddressableBus {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl AddressableBus {
    pub fn new(name: &str) -> Self {
        Self {
            id: ComponentId::new(name),
            state: Arc::new(RwLock::new(AddressableBusState::new())),
        }
    }

    pub fn map(
        &self,
        component: impl AddressableComponent,
        address: usize,
        length: usize,
    ) -> Result<(), AddressableBusError> {
        let range = Range {
            start: address,
            end: address + length,
        };
        {
            let mut state = self.state.write().unwrap();
            state.mappings.insert(range, Arc::new(component));
            for (range, component) in state.mappings.iter() {
                tracing::info!(
                    "\t0x{:04X} - 0x{:04X}: {}",
                    range.start,
                    range.end,
                    component.id()
                );
            }
        }
        Ok(())
    }

    pub fn read(&self, address: usize, length: usize) -> Result<Bytes> {
        let state = self.state.read().unwrap();
        let component = state
            .mappings
            .get(&address)
            .ok_or(AddressableBusError::NoComponentMappedAtAddress(address))?;
        Ok(component.read(address, length)?)
    }

    pub fn write(&self, address: usize, data: &[u8]) -> Result<()> {
        let state = self.state.read().unwrap();
        let component = state
            .mappings
            .get(&address)
            .ok_or(AddressableBusError::NoComponentMappedAtAddress(address))?;
        component.write(address, data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use anyhow::Result;
    use bytes::{BufMut, Bytes, BytesMut};
    use rand::Rng;

    use super::{AddressableBus, AddressableComponent, Component, ComponentId};

    #[derive(Clone)]
    struct TestComponent {
        id: ComponentId,
        data: Arc<RwLock<BytesMut>>,
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }
    }

    impl AddressableComponent for TestComponent {
        fn read(&self, address: usize, length: usize) -> Result<Bytes> {
            let data = self.data.read().unwrap();
            let slice = &data[address..address + length];
            Ok(Bytes::copy_from_slice(slice))
        }

        fn write(&self, address: usize, bytes: &[u8]) -> Result<()> {
            let mut data = self.data.write().unwrap();
            let mut buf = &mut data[address..address + bytes.len()];
            buf.put_slice(bytes);
            Ok(())
        }
    }

    impl TestComponent {
        pub fn new(name: &str) -> Self {
            let mut buf = [0u8; 1024];
            rand::thread_rng().fill(&mut buf);
            let data = BytesMut::from_iter(&buf);
            Self {
                id: ComponentId::new(name),
                data: Arc::new(RwLock::new(data)),
            }
        }
    }

    fn setup() -> ([TestComponent; 3], AddressableBus) {
        let bus = AddressableBus::new("test bus");
        let components = [
            TestComponent::new("a"),
            TestComponent::new("b"),
            TestComponent::new("c"),
        ];
        (components, bus)
    }

    #[test]
    fn new_works() {
        let _ = AddressableBus::new("test bus");
    }

    #[test]
    fn map_works() {
        let ([a, b, c], bus) = setup();
        assert!(!bus.read(0x0000, 4).is_ok());

        bus.map(a, 0x0000, 0x00FF).unwrap();
        bus.map(b, 0x0100, 0x00FF).unwrap();
        bus.map(c, 0x0200, 0x00FF).unwrap();

        assert!(bus.read(0x0000, 4).is_ok());
    }
}
