use std::fmt;
use crate::component::Component;

#[derive(Debug)]
pub struct Error;
pub type Result<T> = core::result::Result<T, Error>;

pub trait OscillatorClient {
    fn tick(&mut self) {}
}

pub struct Oscillator<'a> {
    clients: Vec<&'a mut dyn OscillatorClient>,
    frequency_hz: u64,
}

impl<'a> Component for Oscillator<'a> { }

impl<'a> fmt::Debug for Oscillator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oscillator: {}hz", self.frequency_hz)
    }
}

impl<'a> Oscillator<'a> {
    pub fn new(frequency_hz: u64) -> Self {
        Oscillator {
            clients: Vec::new(),
            frequency_hz,
        }
    }

    pub fn register_client(&mut self, client: &'a mut dyn OscillatorClient) -> Result<()> {
        self.clients.push(client);
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        let freq: f64 = self.frequency_hz as f64;
        let period_secs: f64 = 1.0 / freq;
        println!(
            "starting oscillator with period {} ms",
            period_secs * 1000.0
        );
        let period = std::time::Duration::from_secs_f64(period_secs);
        let mut interval_timer = tokio::time::interval(period);
        let start_instant = tokio::time::Instant::now();

        loop {
            let now_instant = interval_timer.tick().await;
            let total_elapsed = now_instant.duration_since(start_instant);
            let last_tick_elapsed = now_instant.elapsed();
            println!(
                "last tick elapsed: {}ns (total: {} secs)",
                last_tick_elapsed.as_nanos(),
                total_elapsed.as_secs_f32()
            );
            for client in &mut self.clients {
                client.tick();
            }
        }
    }
}
