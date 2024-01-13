#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{Align2, Color32, Stroke};
use threegui::{ThreeUi, Vec3};

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .unwrap(),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(TemplateApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TemplateApp {
    // Example stuff:
    label: String,

    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| threegui::threegui(ui, |three| self.ui_3d(three)));
    }
}

impl TemplateApp {
    fn ui_3d(&mut self, three: &mut ThreeUi) {
        axes(three);
    }
}

fn axes(three: &mut ThreeUi) {
    let paint = three.painter();

    for (dim, color, name) in [
        (Vec3::X, Color32::RED, "X"),
        (Vec3::Y, Color32::GREEN, "Y"),
        (Vec3::Z, Color32::LIGHT_BLUE, "Z"),
    ] {
        paint.line(Vec3::ZERO, dim, Stroke::new(1., color));
        paint.text(
            dim * 1.1,
            Align2::CENTER_CENTER,
            name,
            Default::default(),
            color,
        );
    }
}
