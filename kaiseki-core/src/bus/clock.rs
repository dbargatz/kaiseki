use async_trait::async_trait;

use crate::bus::base::{BaseBus, BaseBusMessage, BaseBusRef};
use crate::component::{Component2, Component2Id};

#[derive(Clone, Debug)]
pub enum ClockBusMessage {
    StartCycle(usize),
}

impl BaseBusMessage for ClockBusMessage {}

pub struct ClockBus {
    id: Component2Id,
    inner_bus: BaseBus<ClockBusMessage>,
}

// C1Ref c1a, c1b, c1c -> C1A { Bus1Ref b1a }
// C2Ref c2a, c2b -> C2A { Bus1Ref b1b, BaseBusRef b2a }
// Bus1Ref b1a, b1b -> B1A { C2Ref c2a }
// BaseBusRef c

#[async_trait]
impl Component2 for ClockBus {
    type Ref = ClockBusRef;

    fn id(&self) -> &Component2Id {
        &self.id
    }

    async fn run(&mut self) {
        self.inner_bus.run().await
    }
}

impl ClockBus {
    pub fn create(id: Component2Id) -> <Self as Component2>::Ref {
        let (inner_bus, ref_sender) = BaseBus::<ClockBusMessage>::create();
        let mut bus = Self { id, inner_bus };
        tokio::spawn(async move { bus.run().await });
        <ClockBus as Component2>::Ref::new(&id, ref_sender)
    }
}

pub type ClockBusRef = BaseBusRef<ClockBusMessage>;

impl ClockBusRef {
    pub async fn start_cycle(&self, cycle_num: usize) {
        self.broadcast(ClockBusMessage::StartCycle(cycle_num)).await
    }
}
