use std::time::Duration;

use async_trait::async_trait;
use tokio::time::Instant;

use crate::bus::ClockBusRef;
use crate::component::{Component2, Component2Id, Component2Ref};

#[derive(Debug)]
struct ClockMetrics {
    start_time: Instant,
    stop_time: Option<Instant>,
}

impl ClockMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            stop_time: None,
        }
    }
}

pub enum Clock2Message {
    StartClock,
    StopClock,
}

#[derive(Debug)]
pub struct Clock2 {
    id: Component2Id,
    bus: ClockBusRef,
    current_cycle: usize,
    frequency_hz: usize,
    metrics: ClockMetrics,
    multiplier: f64,
    ref_receiver: async_channel::Receiver<Clock2Message>,
    running: bool,
}

#[async_trait]
impl Component2 for Clock2 {
    type Ref = Clock2Ref;

    fn id(&self) -> &Component2Id {
        &self.id
    }

    async fn run(&mut self) {
        loop {
            let period_secs = (1.0 / self.frequency_hz as f64) * self.multiplier;
            let period = Duration::from_secs_f64(period_secs);
            let sleep_fut = tokio::time::sleep(period);

            let ref_receiver = self.ref_receiver.clone();
            let receiver_fut = ref_receiver.recv();

            tokio::select! {
                res = receiver_fut, if !ref_receiver.is_closed() => {
                    match res {
                        Ok(Clock2Message::StartClock) => {
                            if self.running {
                                tracing::warn!("cannot start clock, already running");
                                continue
                            }
                            self.running = true;
                            self.metrics.start_time = Instant::now();
                            self.metrics.stop_time = None;
                        },
                        Ok(Clock2Message::StopClock) => {
                            if !self.running {
                                tracing::warn!("cannot stop clock, is not running");
                                continue
                            }
                            self.running = false;
                            self.metrics.stop_time = Some(Instant::now());
                        },
                        Err(err) => {
                            tracing::error!("receive error for open receiver: {}", err);
                        }
                    }
                },
                _ = sleep_fut, if self.running => {
                    self.bus.start_cycle(self.current_cycle).await;
                },
                else => {
                    tracing::info!("component {} (clock) exiting", self.id());
                    self.bus.disconnect().await;
                    break;
                },
            };
        }
    }
}

impl Clock2 {
    pub fn create(
        id: Component2Id,
        bus: &ClockBusRef,
        frequency_hz: usize,
        multiplier: f64,
    ) -> <Clock2 as Component2>::Ref {
        // TODO: how should this channel be bounded?
        let (ref_sender, ref_receiver) = async_channel::bounded(8);
        let mut component = Self {
            id,
            bus: bus.connect(id),
            current_cycle: 0,
            frequency_hz,
            metrics: ClockMetrics::new(),
            multiplier,
            ref_receiver,
            running: false,
        };
        tokio::spawn(async move { component.run().await });
        <Clock2 as Component2>::Ref::new(&id, ref_sender)
    }
}

pub type Clock2Ref = Component2Ref<Clock2Message>;

impl Clock2Ref {
    pub async fn start(&self) {
        self.send(Clock2Message::StartClock).await
    }

    pub async fn stop(&self) {
        self.send(Clock2Message::StopClock).await
    }
}
