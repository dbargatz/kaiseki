use std::fs;
use std::io;

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::{Component, Result};
use tracing_subscriber::prelude::*;

fn main() -> Result<()> {
    let stderr_format = tracing_subscriber::fmt::layer().with_writer(io::stderr);

    tracing_subscriber::registry().with(stderr_format).init();

    tracing::info!("loading Chip-8 program");
    let program = fs::read("kaiseki-chip8/assets/Chip8 Picture.ch8").unwrap();
    let machine = Chip8Machine::new(&program).unwrap();
    machine.start();
    Ok(())
}
