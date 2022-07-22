use std::fs;

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::{Component, Result};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("loading Chip-8 program");
    let program = fs::read("kaiseki-chip8/assets/Chip8 Picture.ch8").unwrap();
    let mut machine = Chip8Machine::new(&program).unwrap();
    machine.start();
    Ok(())
}
