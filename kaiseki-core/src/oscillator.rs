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
    cycles: usize,
    frequency_hz: usize,
}

impl Component for Oscillator {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn start(&mut self) {
        let freq: f64 = self.frequency_hz as f64;
        let period_secs: f64 = 1.0 / freq;
        let period_duration = std::time::Duration::from_secs_f64(period_secs);
        tracing::info!("starting oscillator with period {}ns", period_secs);

        let start_time = std::time::Instant::now();
        let mut period = period_duration;

        loop {
            self.cycles += 1;
            tracing::info!("starting cycle {}", self.cycles);
            let msg = OscillatorBusMessage::CycleStart { cycle_number: self.cycles };
            let period_start = std::time::Instant::now();
            self.bus.send(msg);
            let _ = self.bus.recv().unwrap();
            let period_end = std::time::Instant::now();

            let total_elapsed = period_end - start_time;
            let period_elapsed = period_end - period_start;
            let expected_elapsed = period_duration.mul_f64(self.cycles as f64);
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

            tracing::info!("ending cycle {} | elapsed: {}ns | next: {}ns | total: {}s", self.cycles, period_elapsed.as_nanos(), period.as_nanos(), total_elapsed.as_secs_f32());
            if !diff.is_zero() {
                let percent_lag = 100.0 * (diff.as_secs_f32() / total_elapsed.as_secs_f32());
                tracing::warn!("oscillator is lagging real-time by {}s ({:.2}%)", diff.as_secs_f32(), percent_lag);
            }

            if !period.is_zero() {
                spin_sleep::sleep(period);
            }
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
            cycles: 0,
            frequency_hz,
        }
    }
}
