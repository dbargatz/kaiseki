use anyhow::Result;
use clap::{Parser, ValueEnum};

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::Vex;
use kaiseki_ui::create_ui;
use tracing_flame::FlameLayer;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum SupportedMachines {
    Chip8,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_enum, value_parser, short, long)]
    machine: SupportedMachines,
}

fn create_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn main() -> Result<()> {
    let _guard = config_tracing();

    let args = Args::parse();
    let machine_type = args.machine;
    let guest = match machine_type {
        SupportedMachines::Chip8 => {
            let machine = Chip8Machine::new()?;
            Vex::create(machine, "kaiseki-chip8/assets/Chip8 Picture.ch8")
        }
    };

    let (start_tx, start_rx) = tokio::sync::oneshot::channel::<bool>();
    let ui_guest = guest.clone();

    tracing::info!("starting emulator thread");
    let emulator_thread = std::thread::spawn(move || {
        let runtime = create_tokio_runtime();
        runtime.block_on(async {
            let _ = start_rx.await;
            guest.start().await.unwrap();
        });
    });

    tracing::info!("creating UI");
    if let Err(err) = create_ui(ui_guest, start_tx) {
        tracing::warn!("could not create UI: {}", err);
    };

    tracing::info!("waiting for emulator thread");
    let _ = emulator_thread.join();
    Ok(())
}

#[cfg(not(debug_assertions))]
fn config_tracing() {
    tracing_subscriber::fmt::init();
}

#[cfg(debug_assertions)]
fn config_tracing() -> impl Drop {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};
    use tracing_subscriber::prelude::*;

    let console_layer = console_subscriber::spawn();
    let fmt_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(fmt_filter);

    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();

    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt_layer)
        .with(flame_layer)
        .init();

    _guard
}
