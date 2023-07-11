use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};

use eframe::CreationContext;
use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::Vex;
use tokio::sync::oneshot::Sender;

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

struct KaisekiApp {
    args: Args,
    vex: Vex,
    start_tx: Option<Sender<bool>>,
}

impl eframe::App for KaisekiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        if self.start_tx.is_some() && ctx.frame_nr() > 0 {
            let start_tx = std::mem::take(&mut self.start_tx).unwrap();
            let _ = start_tx.send(true);
        }
        egui::Window::new("Display")
            .collapsible(false)
            .default_size((64.0 * 8.0, 32.0 * 8.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Selected machine: {:?}", self.args.machine));
                ui.label(format!("Frame number: {:?}", ctx.frame_nr()));
                ui.allocate_space(ui.available_size());
            });
    }
}

impl KaisekiApp {
    pub fn new(_creation_ctx: &CreationContext, args: Args, vex: Vex, start_tx: Sender<bool>) -> Self {
        Self { args, vex, start_tx: Some(start_tx) }
    }
}

fn create_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn create_ui(args: Args, vex: Vex, start_tx: Sender<bool>) -> Result<()> {
    let options = eframe::NativeOptions::default();
    let res = eframe::run_native(
        "Kaiseki",
        options,
        Box::new(|cc| Box::new(KaisekiApp::new(cc, args, vex, start_tx))),
    );
    match res {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!("Could not set up graphics context")),
    }
}

fn main() -> Result<()> {
    config_tracing();

    let args = Args::parse();
    let machine_type = args.machine;
    let guest = match machine_type {
        SupportedMachines::Chip8 => {
            let machine = Chip8Machine::new()?;
            Vex::create(machine, "kaiseki-chip8/assets/Chip8 Picture.ch8")
        }
    };

    let (start_tx, start_rx) = tokio::sync::oneshot::channel::<bool>();
    let uiguest = guest.clone();

    tracing::info!("starting emulator thread");
    let emulator_thread = std::thread::spawn(move || {
        let runtime = create_tokio_runtime();
        runtime.block_on(async {
            let _ = start_rx.await;
            guest.start().await.unwrap();
        });
    });

    tracing::info!("creating ui");
    create_ui(args, uiguest, start_tx)?;

    tracing::info!("waiting for emulator thread");
    let _ = emulator_thread.join();
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
