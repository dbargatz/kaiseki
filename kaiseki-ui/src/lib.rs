use egui::{ColorImage, TextureFilter, TextureOptions};
use kaiseki_core::Vex;
use thiserror::Error;
use tokio::sync::oneshot::Sender;

#[derive(Debug, Error)]
pub enum UiError {
    #[error("error creating native UI")]
    CreationError(#[from] eframe::Error),
}

pub type Result<T> = std::result::Result<T, UiError>;

pub struct KaisekiUiApp {
    pub vex: Vex,
    pub start_tx: Option<Sender<bool>>,
}

impl KaisekiUiApp {
    pub fn new(vex: Vex, start_tx: Sender<bool>) -> Self {
        Self {
            vex,
            start_tx: Some(start_tx),
        }
    }
}

impl eframe::App for KaisekiUiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        let (width, height, frame) = self.vex.get_frame();
        let image = ColorImage::from_rgb([width, height], &frame);
        let options = TextureOptions {
            magnification: TextureFilter::Nearest,
            minification: TextureFilter::Nearest,
        };
        let texture = ctx.load_texture("display", image, options);

        egui::Window::new("Kaiseki")
            .collapsible(false)
            .default_size((64.0 * 8.0, 32.0 * 8.0))
            .resizable(false)
            .show(ctx, |ui| {
                ui.image((
                    texture.id(),
                    [width as f32 * 8.0, height as f32 * 8.0].into(),
                ));
                ui.label(format!("Frame number: {:?}", ctx.frame_nr()));
                ui.allocate_space(ui.available_size());
            });

        if self.start_tx.is_some() && ctx.frame_nr() > 0 {
            let start_tx = std::mem::take(&mut self.start_tx).unwrap();
            let _ = start_tx.send(true);
        }
    }
}

pub fn create_ui(vex: Vex, start_tx: Sender<bool>) -> Result<()> {
    tracing::info!("creating native UI");

    let options = eframe::NativeOptions::default();
    let res = eframe::run_native(
        "Kaiseki",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(KaisekiUiApp::new(vex, start_tx))
        }),
    );
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(UiError::CreationError(err)),
    }
}
