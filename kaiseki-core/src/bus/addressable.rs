use std::fmt;
use std::ops::RangeInclusive;
use std::sync::{Arc, RwLock};

use rangemap::RangeInclusiveMap;

use crate::component::{
    AddressableComponent, AddressableComponentError, Component, ComponentId, Result,
};

struct AddressableBusState {
    mappings: RangeInclusiveMap<usize, Arc<dyn AddressableComponent>>,
}

impl AddressableBusState {
    pub fn new() -> Self {
        Self {
            mappings: RangeInclusiveMap::new(),
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
        let state = self.state.read().unwrap();
        f.write_str(format!("{}\n", self.id()).as_str())?;

        if state.mappings.iter().count() == 0 {
            f.write_str("\tno mapped components")?;
            return Ok(());
        }

        for (range, component) in state.mappings.iter() {
            f.write_fmt(format_args!(
                "\t0x{:04X} - 0x{:04X}: {}\n",
                range.start(),
                range.end(),
                component.id()
            ))?;
        }
        Ok(())
    }
}

impl Component for AddressableBus {
    fn id(&self) -> &ComponentId {
        &self.id
    }
}

impl AddressableComponent for AddressableBus {
    fn read(&self, address: usize, length: usize) -> Result<Vec<u8>> {
        tracing::trace!("bus: reading {} bytes from 0x{:08X}", length, address);
        let state = self.state.read().unwrap();
        let (range, component) = state.mappings.get_key_value(&address).ok_or(
            AddressableComponentError::NoComponentMappedAtAddress(address),
        )?;

        let adjusted_address = address - range.start();
        match component.read(adjusted_address, length) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(AddressableComponentError::ComponentReadFailed(
                component.id().clone(),
                address,
                length,
            )),
        }
    }

    fn write(&self, address: usize, data: &[u8]) -> Result<()> {
        tracing::trace!("bus: writing {} bytes to 0x{:08X}", data.len(), address);
        let state = self.state.read().unwrap();
        let (range, component) = state.mappings.get_key_value(&address).ok_or(
            AddressableComponentError::NoComponentMappedAtAddress(address),
        )?;

        let adjusted_address = address - range.start();
        match component.write(adjusted_address, data) {
            Ok(_) => Ok(()),
            Err(_) => Err(AddressableComponentError::ComponentWriteFailed(
                component.id().clone(),
                address,
                data.len(),
            )),
        }
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
        address_range: RangeInclusive<usize>,
        component: impl AddressableComponent,
    ) -> Result<()> {
        let mut state = self.state.write().unwrap();

        // Ensure the new component mapping doesn't overlap with any components already mapped-in.
        for (existing_range, existing_component) in state.mappings.iter() {
            // The mappings.iter() method returns mappings in-order from lowest to
            // highest range start, and ranges are [start, end). This implies that
            // if the existing mapping's range start is greater than the new mapping's
            // range end, the new mapping cannot overlap with the existing mapping, or
            // any following mapping. As a result, if we encounter this condition, we
            // can safely break out of the overlap-checking loop early.
            if existing_range.start() > address_range.end() {
                break;
            }

            // Determine if the start or end address of the existing or new mappings
            // overlap with each other at all. If so, we have a mapping conflict and
            // need to return an error.
            if existing_range.contains(address_range.start())
                || existing_range.contains(address_range.end())
                || address_range.contains(existing_range.start())
                || address_range.contains(existing_range.end())
            {
                return Err(AddressableComponentError::MappingConflict(
                    component.id().clone(),
                    address_range,
                    existing_component.id().clone(),
                ));
            }
        }

        // We've determined that none of the existing mappings conflict with the given
        // new mapping, so go ahead and insert the new mapping!
        state.mappings.insert(address_range, Arc::new(component));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use rand::Rng;

    use super::{
        AddressableBus, AddressableComponent, AddressableComponentError, Component, ComponentId,
        Result,
    };

    #[derive(Clone)]
    struct TestComponent {
        id: ComponentId,
        data: Arc<RwLock<Vec<u8>>>,
    }

    impl Component for TestComponent {
        fn id(&self) -> &ComponentId {
            &self.id
        }
    }

    impl AddressableComponent for TestComponent {
        fn read(&self, address: usize, length: usize) -> Result<Vec<u8>> {
            let data = self.data.read().unwrap();
            let slice = &data[address..address + length];
            Ok(Vec::from(slice))
        }

        fn write(&self, address: usize, bytes: &[u8]) -> Result<()> {
            let mut data = self.data.write().unwrap();
            data.splice(address..address + bytes.len(), bytes.iter().cloned());
            Ok(())
        }
    }

    impl TestComponent {
        pub fn new(name: &str) -> Self {
            let mut buf = [0u8; 1024];
            rand::thread_rng().fill(&mut buf);
            let data = Vec::from(buf);
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
        let ([a, _, _], bus) = setup();
        assert_eq!(
            bus.read(0x0000, 4),
            Err(AddressableComponentError::NoComponentMappedAtAddress(
                0x0000
            ))
        );

        bus.map(0x0000..=0x0FFF, a.clone()).unwrap();
        assert!(bus.read(0x0000, 4).is_ok());
    }

    #[test]
    fn map_prevents_conflicts() {
        let ([a, b, _], bus) = setup();
        let a_range = 0x0100..=0x01FF;
        let a_id = a.id();
        let b_id = b.id();
        bus.map(a_range.clone(), a.clone()).unwrap();

        // Attempt to map a component that overlaps `a`'s range by a single byte at the start.
        let b_range = 0x0000..=0x0100;
        let err = bus
            .map(b_range.clone(), b.clone())
            .expect_err("map() should have failed");
        assert_eq!(
            err,
            AddressableComponentError::MappingConflict(b_id.clone(), b_range, a_id.clone(),)
        );

        // Attempt to map a component that overlaps `a`'s range by a single byte at the end.
        let b_range = 0x01FF..=0x02FF;
        let err = bus
            .map(b_range.clone(), b.clone())
            .expect_err("map() should have failed");
        assert_eq!(
            err,
            AddressableComponentError::MappingConflict(b_id.clone(), b_range, a_id.clone(),)
        );

        // Attempt to map a component that completely contains `a`'s range.
        let b_range = 0x0000..=0x02FF;
        let err = bus
            .map(b_range.clone(), b.clone())
            .expect_err("map() should have failed");
        assert_eq!(
            err,
            AddressableComponentError::MappingConflict(b_id.clone(), b_range, a_id.clone(),)
        );

        // Attempt to map a component that is completely contained within `a`'s range.
        let b_range = 0x0180..=0x018F;
        let err = bus
            .map(b_range.clone(), b.clone())
            .expect_err("map() should have failed");
        assert_eq!(
            err,
            AddressableComponentError::MappingConflict(b_id.clone(), b_range, a_id.clone(),)
        );

        // Attempt to map a component that is exactly `a`'s range.
        let b_range = 0x0100..=0x01FF;
        let err = bus
            .map(b_range.clone(), b.clone())
            .expect_err("map() should have failed");
        assert_eq!(
            err,
            AddressableComponentError::MappingConflict(b_id.clone(), b_range, a_id.clone(),)
        );
    }
}
