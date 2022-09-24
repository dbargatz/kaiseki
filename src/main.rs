use anyhow::Result;
use clap::{ArgEnum, Parser};

use kaiseki_chip8::machine::Chip8Machine;
use kaiseki_core::Vex;
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

struct KaisekiApp {
    args: Args,
}

impl eframe::App for KaisekiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::Window::new("Display")
            .collapsible(false)
            .default_size((64.0 * 8.0, 32.0 * 8.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Selected machine: {:?}", self.args.machine));
                ui.allocate_space(ui.available_size());
            });
    }
}

fn create_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn create_ui(app: KaisekiApp) {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Kaiseki", options, Box::new(|_cc| Box::new(app)));
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
        SupportedMachines::Gameboy => {
            let machine = GameboyMachine::new()?;
            Vex::create(machine, "")
        }
    };

    let emulator_thread = std::thread::spawn(move || {
        let runtime = create_tokio_runtime();
        runtime.block_on(async {
            guest.start().await.unwrap();
        });
    });
    let app = KaisekiApp { args };
    create_ui(app);
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
