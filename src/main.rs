#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::time::Instant;

use egui::{Align2, Color32, ComboBox, DragValue, ScrollArea, Stroke, Ui, WidgetText};
use quantum::{b_field, spin_expectation, Complex, SpinState, SZ_POSITIVE_STATE};
use threegui::{utils, ThreeUi, Vec3};

mod quantum;

fn is_mobile(ctx: &egui::Context) -> bool {
    use egui::os::OperatingSystem;
    matches!(ctx.os(), OperatingSystem::Android | OperatingSystem::IOS)
}

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
        "Spin experiment for PH451",
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

    trace: bool,
    tracing: Vec<Vec3>,
    max_trace_points: usize,

    show_psi_plot: bool,
    increment_angle: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            b_field_strength: 0.9,
            theta: 0.17,
            initial_state: quantum::SZ_POSITIVE_STATE,
            time: 0.,

            play: true,
            anim_speed: 1.,

            trace: true,
            tracing: vec![],

            show_psi_plot: true,
            increment_angle: false,
            max_trace_points: 600,
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
            let delta = ctx.input(|r| r.stable_dt) * self.anim_speed;
            if self.increment_angle {
                self.theta += delta;
            } else {
                self.time += delta;
            }
        }

        if self.trace {
            let spin_vector: mint::Vector3<f32> = spin_expectation(
                self.theta,
                self.initial_state,
                self.b_field_strength,
                self.time,
            )
            .into();

            if self.tracing.len() > self.max_trace_points {
                let idx = self
                    .tracing
                    .len()
                    .checked_sub(self.max_trace_points)
                    .unwrap_or(0);
                self.tracing = self.tracing[idx..].to_vec();
            }

            self.tracing.push(spin_vector.into());
        } else {
            self.tracing.clear();
        }

        if is_mobile(ctx) {
            egui::TopBottomPanel::bottom("panel").show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| self.settings_panel(ui))
                });
            });
        } else {
            egui::SidePanel::left("panel").show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| self.settings_panel(ui))
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("3D plot");
            threegui::threegui(ui, |three| self.ui_3d(three));
            if self.show_psi_plot {
                self.plot_psi(ui);
            }
        });
    }
}

impl TemplateApp {
    fn ui_3d(&mut self, three: &mut ThreeUi) {
        // Draw grid
        utils::grid(
            three.painter(),
            10,
            1.,
            Stroke::new(1.0, Color32::from_gray(45)),
        );

        // Draw axes
        axes(three);

        // Draw B field
        let b_field: mint::Vector3<f32> = b_field(self.theta, self.b_field_strength).into();
        label_line(three, b_field.into(), Color32::from_rgb(222, 230, 44), "B");

        // Draw spin vector
        let spin_vector: mint::Vector3<f32> = spin_expectation(
            self.theta,
            self.initial_state,
            self.b_field_strength,
            self.time,
        )
        .into();
        label_line(three, spin_vector.into(), Color32::LIGHT_BLUE, "<S>");

        // Draw tracing
        let paint = three.painter();
        for pair in self.tracing.windows(2) {
            paint.line(pair[0], pair[1], Stroke::new(1., Color32::LIGHT_BLUE));
        }

        /*
        let mut projected = spin_vector;
        projected.y = 0.;
        three.painter().line(
        spin_vector.into(),
        projected.into(),
        Stroke::new(1.0, Color32::DARK_GRAY),
        );
        */
    }

    fn psi(&self) -> SpinState {
        quantum::psi(
            self.theta,
            self.initial_state,
            self.b_field_strength,
            self.time,
        )
    }

    fn plot_psi(&mut self, ui: &mut Ui) {
        ui.label("Wavefunction components");
        let psi = self.psi();
        egui_plot::Plot::new("psi")
            .width(300.)
            .include_y(-1.0)
            .include_y(1.0)
            .include_x(-1.0)
            .include_x(1.0)
            .view_aspect(1.0)
            //.data_aspect(1.0)
            //.center_x_axis(true)
            //.center_y_axis(true)
            //.height(200.)
            .show(ui, |plot| {
                for (cpx, color, name) in [
                    (psi.x, Color32::LIGHT_BLUE, "a"),
                    (psi.y, Color32::RED, "b"),
                ] {
                    let cpx = [cpx.re, cpx.im].map(|v| v as f64);
                    let points = [[0., 0.], cpx];
                    plot.line(
                        egui_plot::Line::new(egui_plot::PlotPoints::new(points.to_vec()))
                            .color(color),
                    );
                    plot.text(
                        egui_plot::Text::new(
                            cpx.into(),
                            WidgetText::from(name).strong().color(color),
                        )
                        .anchor(Align2::LEFT_CENTER),
                    );
                }
            });
    }

    fn settings_panel(&mut self, ui: &mut Ui) {
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
                .speed(1e-2),
        );

        /*
        ui.strong("Initial state");
        let speed = 1e-2;
        edit_complex(ui, &mut self.initial_state.x, "a: ", speed);
        edit_complex(ui, &mut self.initial_state.y, "b: ", speed);
        */

        ui.separator();
        ui.strong("Animation");
        ui.checkbox(&mut self.play, "Play animation");
        ui.add(
            DragValue::new(&mut self.anim_speed)
                .prefix("Speed: ")
                .suffix("x")
                .speed(1e-2),
        );
        ui.checkbox(&mut self.increment_angle, "Animate θ (constant t)");

        ui.separator();
        ui.strong("Internals");
        let psi = self.psi();
        ui.label("Spin wave function ψ");
        ui.label(format!("a = {:.02}", psi.x));
        ui.label(format!("b = {:.02}", psi.y));

        ui.separator();
        ui.strong("Visualization");
        ui.checkbox(&mut self.trace, "Trace spin vector");
        ui.checkbox(&mut self.show_psi_plot, "Show complex plane");

        ui.separator();
        ui.strong("Shortcuts");
        ui.horizontal(|ui| {
            if ui.button("θ -=  π/4").clicked() {
                self.theta -= std::f32::consts::FRAC_PI_4;
            }
            if ui.button("θ +=  π/4").clicked() {
                self.theta += std::f32::consts::FRAC_PI_4;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("θ -=  π/2").clicked() {
                self.theta -= std::f32::consts::FRAC_PI_2;
            }
            if ui.button("θ +=  π/2").clicked() {
                self.theta += std::f32::consts::FRAC_PI_2;
            }
        });

        if ui.button("Zero angle").clicked() {
            self.theta = 0.;
        }

        ui.horizontal(|ui| {
            if ui.button("Reset time").clicked() {
                self.time = Self::default().time;
            }

            if ui.button("Reset angle").clicked() {
                self.theta = Self::default().theta;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Reset trace").clicked() {
                self.tracing.clear();
            }

            if ui.button("Reset all").clicked() {
                *self = Self::default();
            }
        });

        if ui.button("Fast preset").clicked() {
            self.anim_speed = 15.0;
            self.max_trace_points = 100;
        }

        ui.add(DragValue::new(&mut self.max_trace_points).prefix("Maximum traced points: "));

        // TODO: Normalize button
    }
}

fn axes(three: &mut ThreeUi) {
    label_line(three, Vec3::X, Color32::from_rgb(236, 52, 28), "X");
    label_line(three, Vec3::Y, Color32::from_rgb(85, 230, 33), "Y");
    label_line(three, Vec3::Z, Color32::from_rgb(28, 112, 232), "Z");
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
