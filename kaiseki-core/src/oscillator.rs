use std::fmt;

use async_trait::async_trait;

use crate::bus::Bus;
use crate::component::{Component, ComponentId};
use crate::BusMessage;

#[derive(Clone, Debug)]
pub enum OscillatorBusMessage {
    CycleBatchStart {
        start_cycle: usize,
        cycle_budget: usize,
    },
    CycleBatchEnd {
        start_cycle: usize,
        cycles_spent: usize,
    },
}

impl BusMessage for OscillatorBusMessage {}

pub type OscillatorBus = Bus<OscillatorBusMessage>;

pub struct Oscillator {
    id: ComponentId,
    bus: OscillatorBus,
    frequency_hz: f64,
    period: std::time::Duration,
}

#[async_trait]
impl Component for Oscillator {
    fn id(&self) -> ComponentId {
        self.id
    }

    async fn start(&mut self) {
        tracing::info!(
            "starting oscillator with frequency {}hz / period {}ns",
            self.frequency_hz,
            self.period.as_nanos()
        );

        let start_time = tokio::time::Instant::now();
        let mut current_period = self.period;
        let mut next_period = self.period;
        let mut current_cycle: usize = 0;
        let mut cycle_budget: usize = self.frequency_hz as usize;

        loop {
            tracing::info!(
                "starting cycles {} - {}",
                current_cycle,
                current_cycle + cycle_budget
            );
            let start_msg = OscillatorBusMessage::CycleBatchStart {
                start_cycle: current_cycle,
                cycle_budget,
            };

            let mut cycles_executed: usize = 0;
            let mut end_cycle: usize = 0;

            let period_start = tokio::time::Instant::now();
            self.bus.send(&self.id, start_msg).await.unwrap();
            if let Ok(OscillatorBusMessage::CycleBatchEnd {
                start_cycle,
                cycles_spent,
            }) = self.bus.recv(&self.id).await
            {
                assert!(current_cycle == start_cycle);
                assert!(cycles_spent > 0);
                cycles_executed = cycles_spent;
                end_cycle = start_cycle + cycles_executed;
                match cycles_spent.cmp(&cycle_budget) {
                    std::cmp::Ordering::Less => {
                        tracing::info!(
                            "budgeted {} cycles, used {} cycles",
                            cycle_budget,
                            cycles_spent
                        );
                        cycle_budget = cycles_spent;
                    }
                    std::cmp::Ordering::Greater => {
                        tracing::error!(
                            "budgeted {} cycles, but {} were consumed",
                            cycle_budget,
                            cycles_spent
                        );
                    }
                    _ => {}
                }
            }
            let period_end = tokio::time::Instant::now();

            let total_actual_elapsed = period_end - start_time;
            let total_expected_elapsed = self.period.mul_f64(end_cycle as f64);
            let total_multiplier =
                total_actual_elapsed.as_secs_f64() / total_expected_elapsed.as_secs_f64();

            let period_actual_elapsed = period_end - period_start;
            let period_actual_millis = period_actual_elapsed.as_secs_f64() * 1000.0;
            let period_expected_elapsed = self.period.mul_f64(cycles_executed as f64);
            let period_expected_millis = period_expected_elapsed.as_secs_f64() * 1000.0;
            let period_multiplier = period_actual_millis / period_expected_millis;

            let cycle_actual_average: f64 =
                period_actual_elapsed.as_nanos() as f64 / cycles_executed as f64;
            let cycle_expected_average: f64 =
                period_expected_elapsed.as_nanos() as f64 / cycles_executed as f64;
            let cycle_multiplier = cycle_actual_average / cycle_expected_average;

            let total_difference = total_actual_elapsed.saturating_sub(total_expected_elapsed);
            match total_expected_elapsed.cmp(&total_actual_elapsed) {
                std::cmp::Ordering::Less => {
                    next_period = self.period.saturating_sub(total_difference);
                }
                std::cmp::Ordering::Greater => {
                    next_period = self.period + (total_expected_elapsed - total_actual_elapsed);
                }
                std::cmp::Ordering::Equal => {}
            }

            let current_period_millis = current_period.as_secs_f64() * 1000.0;
            let next_period_millis = next_period.as_secs_f64() * 1000.0;

            tracing::info!("ending cycles {} - {}:", current_cycle, end_cycle);
            tracing::info!(
                "\tcycle:   {:.3}ns avg, {:.3}ns expected ({:.2}x slower)",
                cycle_actual_average,
                cycle_expected_average,
                cycle_multiplier
            );
            tracing::info!(
                "\tperiod:  {:.3}ms elapsed, {:.3}ms expected ({:.2}x slower)",
                period_actual_millis,
                period_expected_millis,
                period_multiplier
            );
            tracing::info!(
                "\toverall: {:.3}s elapsed, {:.3}s expected ({:.2}x slower)",
                total_actual_elapsed.as_secs_f64(),
                total_expected_elapsed.as_secs_f64(),
                total_multiplier
            );
            tracing::info!(
                "\tadjust:  {:.3}ms last, {:.3}ms next",
                current_period_millis,
                next_period_millis,
            );

            if total_multiplier > 1.01 && current_cycle % 100_000 == 0 {
                tracing::warn!(
                    "oscillator is lagging real-time by {:.3}s ({:.2}x slower)",
                    total_difference.as_secs_f64(),
                    total_multiplier,
                );
            }

            current_cycle += cycles_executed;
            tokio::time::sleep(next_period).await;
            current_period = next_period;
        }
    }
}

impl fmt::Debug for Oscillator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oscillator: {}hz", self.frequency_hz)
    }
}

impl Oscillator {
    pub fn new(bus: &OscillatorBus, frequency_hz: usize) -> Self {
        let id = ComponentId::new_v4();
        let freq = frequency_hz as f64;
        let period_duration = std::time::Duration::from_secs_f64(1.0 / freq);
        Oscillator {
            id,
            bus: bus.clone(),
            frequency_hz: freq,
            period: period_duration,
        }
    }
}
