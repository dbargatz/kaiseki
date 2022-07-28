use crate::bus::{Bus, BusConnection};
use crate::component::{Component, ComponentId};
use crate::BusMessage;
use std::fmt;

#[derive(Clone, Debug)]
pub enum OscillatorBusMessage {
    CycleStart { cycle_number: usize },
    CycleEnd { cycle_number: usize },
}

impl BusMessage for OscillatorBusMessage {}

pub type OscillatorBus = Bus<OscillatorBusMessage>;

pub struct Oscillator {
    id: ComponentId,
    bus: BusConnection<OscillatorBusMessage>,
    frequency_hz: usize,
}

impl Component for Oscillator {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&self) {
        let freq: f64 = self.frequency_hz as f64;
        let period_secs: f64 = 1.0 / freq;
        let period_duration = std::time::Duration::from_secs_f64(period_secs);
        tracing::info!("starting oscillator with period {}ns", period_secs);

        let start_time = std::time::Instant::now();
        let mut period = period_duration;
        let mut cycles: usize = 0;

        loop {
            cycles += 1;
            tracing::info!("starting cycle {}", cycles);
            let msg = OscillatorBusMessage::CycleStart {
                cycle_number: cycles,
            };
            let period_start = std::time::Instant::now();
            self.bus.send(msg);
            let _ = self.bus.recv().unwrap();
            let period_end = std::time::Instant::now();

            let total_elapsed = period_end - start_time;
            let period_elapsed = period_end - period_start;
            let expected_elapsed = period_duration.mul_f64(cycles as f64);
            let diff = total_elapsed.saturating_sub(expected_elapsed);
            match expected_elapsed.cmp(&total_elapsed) {
                std::cmp::Ordering::Less => {
                    period = period_duration.saturating_sub(diff);
                }
                std::cmp::Ordering::Greater => {
                    period = period_duration + (expected_elapsed - total_elapsed);
                }
                std::cmp::Ordering::Equal => {}
            }

            tracing::info!(
                "ending cycle {} | elapsed: {}ns | next: {}ns | total: {:.3}s | expected: {:.3}s",
                cycles,
                period_elapsed.as_nanos(),
                period.as_nanos(),
                total_elapsed.as_secs_f32(),
                expected_elapsed.as_secs_f32(),
            );

            let multiplier = total_elapsed.as_secs_f64() / expected_elapsed.as_secs_f64();
            if multiplier > 1.01 {
                tracing::warn!(
                    "oscillator is lagging real-time by {:.2}s ({:.2}x slower)",
                    diff.as_secs_f32(),
                    multiplier,
                );
            }

            spin_sleep::sleep(period);
        }
    }
}

impl fmt::Debug for Oscillator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oscillator: {}hz", self.frequency_hz)
    }
}

impl Oscillator {
    pub fn new(bus: &mut Bus<OscillatorBusMessage>, frequency_hz: usize) -> Self {
        let id = ComponentId::new_v4();
        let conn = bus.connect(&id);
        Oscillator {
            id,
            bus: conn,
            frequency_hz,
        }
    }
}
