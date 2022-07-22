use crate::bus::BusConnection;
use crate::component::Component;
use std::fmt;

#[derive(Debug)]
pub enum OscillatorError {
    Unknown,
}

pub type Result<T> = std::result::Result<T, OscillatorError>;

pub struct Oscillator {
    bus: Option<BusConnection>,
    cycles: u64,
    frequency_hz: u64,
}

impl Component for Oscillator {
    fn connect_to_bus(&mut self, bus: BusConnection) {
        self.bus = Some(bus);
    }

    fn start(&mut self) {
        let freq: f64 = self.frequency_hz as f64;
        let period_secs: f64 = 1.0 / freq;
        let period_duration = std::time::Duration::from_secs_f64(period_secs);
        println!("starting oscillator with period {}ns", period_secs);

        let start_time = std::time::Instant::now();
        let mut period = period_duration;

        let bus = self.bus.as_mut().unwrap();

        loop {
            bus.tick(self.cycles);
            let _ = bus.recv().unwrap();

            let period_start = std::time::Instant::now();
            std::thread::sleep(period);
            let period_end = std::time::Instant::now();
            self.cycles += 1;

            let total_elapsed = period_end - start_time;
            let period_elapsed = period_end - period_start;
            let expected_elapsed = period_duration.mul_f64(self.cycles as f64);
            if expected_elapsed > total_elapsed {
                period = period_duration + (expected_elapsed - total_elapsed);
            } else if expected_elapsed < total_elapsed {
                let diff = total_elapsed - expected_elapsed;
                period = period_duration.saturating_sub(diff);
            }

            println!(
                "last tick elapsed: {}ns (total: {} secs)",
                period_elapsed.as_nanos(),
                total_elapsed.as_secs_f32()
            );
        }
    }
}

impl fmt::Debug for Oscillator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oscillator: {}hz", self.frequency_hz)
    }
}

impl Oscillator {
    pub fn new(frequency_hz: u64) -> Self {
        Oscillator {
            bus: None,
            cycles: 0,
            frequency_hz,
        }
    }
}
