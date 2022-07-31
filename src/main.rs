use std::fs;

use clap::{ArgEnum, Parser};

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::Component;
use kaiseki_gameboy::machine::GameboyMachine;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum SupportedMachines {
    Chip8,
    Gameboy,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(arg_enum, value_parser, short, long)]
    machine: SupportedMachines,
}

#[tokio::main]
async fn main() -> kaiseki_core::Result<()> {
    config_tracing();

    let args = Args::parse();
    match args.machine {
        SupportedMachines::Chip8 => {
            tracing::info!("loading Chip-8 program");
            let program = fs::read("kaiseki-chip8/assets/Chip8 Picture.ch8").unwrap();
            let mut machine = Chip8Machine::new(&program).await.unwrap();
            machine.start().await;
        }
        SupportedMachines::Gameboy => {
            tracing::info!("loading gameboy program");
            let mut machine = GameboyMachine::new().await.unwrap();
            machine.start().await;
        }
    }

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
    let fmt_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(fmt_filter);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .init();
}
