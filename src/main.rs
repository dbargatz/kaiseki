use std::fs;

use kaiseki_core::{Chip8Machine, Machine, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let program = fs::read("/Users/dylan/Downloads/Chip8 Picture.ch8").unwrap();
    let mut machine = Chip8Machine::new(&program).unwrap();
    machine.start().await?;
    Ok(())
}
