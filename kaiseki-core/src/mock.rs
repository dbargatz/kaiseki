use std::time::Duration;

use async_trait::async_trait;

use crate::bus::{ClockBusMessage, ClockBusRef};
use crate::component::{Component2, Component2Id, Component2Ref};

pub enum MockMessage {}

#[derive(Debug)]
pub struct Mock {
    id: Component2Id,
    bus: ClockBusRef,
    delay_secs: f64,
}

#[async_trait]
impl Component2 for Mock {
    type Ref = MockRef;

    fn id(&self) -> &Component2Id {
        &self.id
    }

    async fn run(&mut self) {
        loop {
            match self.bus.listen().await {
                ClockBusMessage::StartCycle(cycle_num) => {
                    tracing::info!(
                        "sleeping for {:0.5}s for cycle {}",
                        self.delay_secs,
                        cycle_num
                    );
                    tokio::time::sleep(Duration::from_secs_f64(self.delay_secs)).await;
                }
            }
        }
    }
}

impl Mock {
    pub fn create(
        id: Component2Id,
        bus: &ClockBusRef,
        delay_secs: f64,
    ) -> <Mock as Component2>::Ref {
        // TODO: how should this channel be bounded?
        let (ref_sender, ref_receiver) = async_channel::bounded(1);
        let mut component = Self {
            id,
            bus: bus.connect(id),
            delay_secs,
        };
        tokio::spawn(async move { component.run().await });
        <Mock as Component2>::Ref::new(&id, ref_receiver, ref_sender)
    }
}

pub type MockRef = Component2Ref<MockMessage>;
