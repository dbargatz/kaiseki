use crate::bus::base::{BaseBus, BaseBusMessage, BaseBusRef};

#[derive(Clone, Debug)]
pub enum ClockBusMessage {
    StartCycle(usize),
}

impl BaseBusMessage for ClockBusMessage {}

// C1Ref c1a, c1b, c1c -> C1A { Bus1Ref b1a }
// C2Ref c2a, c2b -> C2A { Bus1Ref b1b, BaseBusRef b2a }
// Bus1Ref b1a, b1b -> B1A { C2Ref c2a }
// BaseBusRef c

pub type ClockBus = BaseBus<ClockBusMessage>;
pub type ClockBusRef = BaseBusRef<ClockBusMessage>;

impl ClockBusRef {
    pub async fn start_cycle(&self, cycle_num: usize) {
        self.broadcast(ClockBusMessage::StartCycle(cycle_num)).await
    }
}
