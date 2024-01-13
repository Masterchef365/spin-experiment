#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Instant;

use egui::{Align2, Color32, DragValue, Stroke, Ui};
use quantum::{b_field, spin_expectation, Complex, SpinState, SZ_POSITIVE_STATE};
use threegui::{ThreeUi, Vec3};

mod quantum;

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
    theta: f32,
    initial_state: SpinState,
    b_field_strength: f32,
    time: f32,
    play: bool,
    anim_speed: f32,

    delta_time: Instant,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            b_field_strength: 1.0,
            theta: 0.,
            initial_state: quantum::SZ_POSITIVE_STATE,
            time: 0.,

            play: false,
            anim_speed: 1.,

            delta_time: Instant::now(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

fn edit_complex(ui: &mut Ui, cpx: &mut Complex, name: &str, speed: f32) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(DragValue::new(&mut cpx.re).prefix("Re: ").speed(speed));
        ui.add(DragValue::new(&mut cpx.im).prefix("Im: ").speed(speed));
    });
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.play {
            ctx.request_repaint();
            self.time += self.delta_time.elapsed().as_secs_f32() * self.anim_speed;
        }
        self.delta_time = Instant::now();

        egui::SidePanel::left("panel").show(ctx, |ui| {
            ui.strong("Parameters");
            ui.add(DragValue::new(&mut self.time).prefix("Time: ").speed(1e-2));
            ui.add(
                DragValue::new(&mut self.theta)
                    .prefix("Angle θ: ")
                    .suffix(" rads")
                    .speed(1e-2),
            );
            ui.add(
                DragValue::new(&mut self.b_field_strength)
                    .prefix("Field strength: ")
                    .suffix(" rads")
                    .speed(1e-2),
            );

            ui.strong("Initial state");
            let speed = 1e-2;
            edit_complex(ui, &mut self.initial_state.x, "a: ", speed);
            edit_complex(ui, &mut self.initial_state.y, "b: ", speed);

            ui.strong("Animation");
            ui.checkbox(&mut self.play, "Play animation");
            ui.add(
                DragValue::new(&mut self.anim_speed)
                    .prefix("Speed: ")
                    .suffix("x")
                    .speed(1e-2),
            );


            // TODO: Normalize button
        });

        egui::CentralPanel::default()
            .show(ctx, |ui| threegui::threegui(ui, |three| self.ui_3d(three)));
    }
}

impl TemplateApp {
    fn ui_3d(&mut self, three: &mut ThreeUi) {
        axes(three);
        let b_field: mint::Vector3<f32> = b_field(self.theta, self.b_field_strength).into();
        label_line(three, b_field.into(), Color32::YELLOW, "B");

        let spin_vector: mint::Vector3<f32> = spin_expectation(
            self.theta,
            self.initial_state,
            self.b_field_strength,
            self.time,
        )
        .into();
        label_line(
            three,
            spin_vector.into(),
            Color32::from_rgb(126, 18, 182),
            "<S>",
        );
    }
}

fn axes(three: &mut ThreeUi) {
    label_line(three, Vec3::X, Color32::RED, "X");
    label_line(three, Vec3::Y, Color32::GREEN, "Y");
    label_line(three, Vec3::Z, Color32::LIGHT_BLUE, "Z");
}

fn label_line(three: &mut ThreeUi, v: Vec3, color: Color32, name: &str) {
    let paint = three.painter();
    paint.line(Vec3::ZERO, v, Stroke::new(1., color));
    paint.text(
        v.normalize() * (v.length() + 0.1),
        Align2::CENTER_CENTER,
        name,
        Default::default(),
        color,
    );
}
