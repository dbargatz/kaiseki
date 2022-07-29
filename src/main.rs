use std::fs;

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::{Component, Result};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    config_tracing();

    tracing::info!("loading Chip-8 program");
    let program = fs::read("kaiseki-chip8/assets/Chip8 Picture.ch8").unwrap();
    let mut machine = Chip8Machine::new(&program).unwrap();
    machine.start().await;

    Ok(())
}

#[cfg(not(debug_assertions))]
fn config_tracing() {
    tracing_subscriber::fmt::init();
}

#[cfg(debug_assertions)]
fn config_tracing() {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};
    use tracing_subscriber::prelude::*;

    let console_layer = console_subscriber::spawn();
    let fmt_filter = EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(fmt_filter);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();
}
