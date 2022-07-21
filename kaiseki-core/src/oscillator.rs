use std::fmt;
use crate::bus::BusConnection;
use crate::component::Component;

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
        println!("starting oscillator with period {}ns", period_secs);

        let start_time = std::time::Instant::now();
        let period = std::time::Duration::from_secs_f64(period_secs);

        let bus = self.bus.as_mut().unwrap();

        loop {
            bus.tick(self.cycles);
            let _ = bus.recv().unwrap();

            let period_start = std::time::Instant::now();
            std::thread::sleep(period);
            self.cycles += 1;
            let period_end = std::time::Instant::now();
            let total_elapsed = period_end - start_time;
            let period_elapsed = period_end - period_start;

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
