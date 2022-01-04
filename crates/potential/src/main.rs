mod axis;
mod renderer;

use anyhow::Result;
use archie::{egui, wgpu};
use particle::Particle;
use poml::parser::ast;

use axis::*;
use renderer::Renderer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {
    renderer: Renderer,
    width: u32,
    height: u32,
    particles: Vec<Particle>,
    page: Page,
    editor_text: String,
    recompiled: bool,
    mouse: glam::Vec2,
    x_axis: Axis,
    y_axis: Axis,
    settings_open: bool,
}

impl App {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        Ok(App {
            renderer: Renderer::new(ctx)?,
            width: WIDTH,
            height: HEIGHT,
            particles: Vec::new(),
            page: Page::Visualiser,
            editor_text: String::new(),
            recompiled: false,
            mouse: glam::Vec2::ZERO,
            x_axis: Axis::new(-1.0, 1.0),
            y_axis: Axis::new(-1.0, 1.0),
            settings_open: false,
        })
    }

    pub fn compile(&mut self) {
        let parse = poml::compile(&self.editor_text);

        match parse {
            Ok(parse) => {
                let root = parse.root();

                self.particles.clear();
                let mut variables = std::collections::HashMap::new();

                for s in root.stmts() {
                    match s.kind() {
                        ast::StmtKind::Shape(shape) => {
                            if let Some(value) = shape.value().map(|v| v.value()) {
                                let name = shape.label().text().unwrap();
                                variables.insert(name, value);
                            }
                        }
                        ast::StmtKind::Object(object) => {
                            let mut params = object.params().unwrap();
                            let value = params.next_value().unwrap();
                            let x = params.next_value().unwrap();
                            let y = params.next_value().unwrap();
                            let label = params.next_name().unwrap();

                            if let Some(&radius) = variables.get(&label.text()) {
                                self.particles.push(Particle::new(
                                    value.value(),
                                    radius,
                                    glam::Vec2::new(x.value(), y.value()),
                                ));
                            }
                        }
                    }
                }
                self.recompiled = true;
            }
            Err(errors) => {
                // there was an error, print it out for now
                eprintln!("Errors");
                for e in errors {
                    eprintln!("{}", e);
                }
            }
        }
    }
}

impl App {
    fn map_pos(&self, pos: glam::Vec2) -> glam::Vec2 {
        // [0, a]
        // [0, 1]       (/a)
        // [0, c-b]     (*(c-b))
        // [b, c]       + b
        fn map(x: f32, a: f32, b: f32, c: f32) -> f32 {
            (x / a) * (c - b) + b
        }
        let (x, y) = (pos.x, pos.y);
        let x = map(x, self.width as f32, self.x_axis.min(), self.x_axis.max());
        let y = map(y, self.height as f32, self.y_axis.max(), self.y_axis.min());
        glam::Vec2::new(x, y)
    }
}

#[derive(PartialEq, Eq)]
enum Page {
    Visualiser,
    Editor,
}

impl archie::event::EventHandler for App {
    fn update(&mut self, ctx: &archie::Context, _dt: f32) {
        self.width = ctx.width();
        self.height = ctx.height();

        if self.recompiled {
            println!("Recompiled");
            self.recompiled = false;
        }
    }

    fn draw(&mut self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        if self.page == Page::Visualiser {
            self.renderer.render(encoder, target)
        }
    }

    fn ui(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.label("Potential");
                ui.separator();

                if ui
                    .selectable_label(self.page == Page::Visualiser, "Visualiser")
                    .clicked()
                {
                    self.page = Page::Visualiser;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#visualiser");
                    }
                }
                if ui
                    .selectable_label(self.page == Page::Editor, "Editor")
                    .clicked()
                {
                    self.page = Page::Editor;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#editor");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.button("🔧").on_hover_text("Settings").clicked() {
                        self.settings_open = !self.settings_open;
                    }
                })
            })
        });

        egui::Window::new("Settings")
            .open(&mut self.settings_open)
            .show(ctx, |ui| {
                ui.label("X axis");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.x_axis.min)
                            .clamp_range(-f32::INFINITY..=self.x_axis.max),
                    );
                    ui.label("≤ X ≤");
                    ui.add(
                        egui::DragValue::new(&mut self.x_axis.max)
                            .clamp_range(self.x_axis.min..=f32::INFINITY),
                    );
                });
                ui.label("Y axis");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.y_axis.min)
                            .clamp_range(-f32::INFINITY..=self.y_axis.max),
                    );
                    ui.label("≤ Y ≤");
                    ui.add(
                        egui::DragValue::new(&mut self.y_axis.max)
                            .clamp_range(self.y_axis.min..=f32::INFINITY),
                    );
                });
            });

        match self.page {
            Page::Editor => {
                egui::TopBottomPanel::bottom("editor_bottom").show(ctx, |ui| {
                    let layout =
                        egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
                    ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                        if ui.button("Compile").clicked() {
                            self.compile();
                        }
                    })
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    egui::SidePanel::right("overview_panel")
                        .min_width(300.0)
                        .show_inside(ui, |ui| {
                            ui.heading("Objects");
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (i, obj) in self.particles.iter().enumerate() {
                                    egui::CollapsingHeader::new(i.to_string())
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            ui.monospace(format!("radius: {:?}", obj.radius));
                                            ui.monospace(format!("value: {}", obj.value));
                                            ui.monospace(format!(
                                                "pos: {{ x: {}, y: {} }}",
                                                obj.pos.x, obj.pos.y
                                            ));
                                        });
                                }
                            });
                        });

                    let editor = egui::TextEdit::multiline(&mut self.editor_text)
                        .desired_width(f32::INFINITY)
                        .desired_rows(50)
                        .code_editor();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add(editor);
                        })
                });
            }
            Page::Visualiser => {
                egui::Window::new("Info").resizable(false).show(ctx, |ui| {
                    ui.small("Under cursor");
                    ui.monospace(format!("pos: {:.2}, {:.2}", self.mouse.x, self.mouse.y));
                    let particle = Particle::new(0.1, 0.1, glam::vec2(0.0, 0.0));
                    let coord = self.mouse;
                    let d = particle.dist(coord);
                    let v = match particle.potential(coord) {
                        Ok(x) => x,
                        Err(x) => x,
                    };
                    let v = v.abs().clamp(0.0, 1.0);
                    let e = particle.force(coord).unwrap_or(0.0);
                    ui.monospace(format!("distance (m): {}", d));
                    ui.monospace(format!("potential (J/C): {}", v));
                    ui.monospace(format!("force (N/C): {}", e));
                });
            }
        }
    }

    fn mouse_moved(&mut self, x: f64, y: f64) {
        let pos = glam::Vec2::new(x as f32, y as f32);
        self.mouse = self.map_pos(pos);
    }

    fn wheel_moved(&mut self, _dx: f32, dy: f32) {
        if self.page == Page::Editor {
            return;
        }
        // zoom into a point
        const ZOOM_INTENSITY: f32 = 0.1;
        // keep delta normalised
        let dy = dy.clamp(-1.0, 1.0);
        let zoom = (dy * ZOOM_INTENSITY).exp();
        // scale axis
        self.x_axis *= zoom;
        self.y_axis *= zoom;
        // translate to keep mouse at the same x and y
        self.x_axis -= self.mouse.x / zoom - self.mouse.x;
        self.y_axis -= self.mouse.y / zoom - self.mouse.y;
    }
}

async fn run() {
    archie::log::init();

    let builder = archie::Context::builder()
        .title("Potential")
        .width(WIDTH)
        .height(HEIGHT)
        .fullscreen(cfg!(target_arch = "wasm32")); // fullscreen for wasm

    let features = wgpu::Features::SPIRV_SHADER_PASSTHROUGH | wgpu::Features::PUSH_CONSTANTS;
    match builder.build(Some(features)).await {
        Ok((event_loop, mut ctx)) => match App::new(&mut ctx) {
            Ok(app) => archie::event::run(ctx, event_loop, app),
            Err(e) => {
                eprintln!("failed to make app; {:?}", e);
            }
        },
        Err(e) => {
            eprintln!("failed to build potential context: {:?}", e);
        }
    }
}

fn main() {
    archie::block_on(run());
}
