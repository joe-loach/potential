mod renderer;

use anyhow::Result;
use archie::{
    egui,
    wgpu::{self, util::DeviceExt},
    winit::event::{ModifiersState, MouseButton, VirtualKeyCode},
};
use glam::{vec2, Vec2};
use particle::Particle;
use poml::parser::ast;

use common::*;
use renderer::Renderer;

struct App {
    time: f32,
    renderer: Renderer,
    width: u32,
    height: u32,
    particles: Vec<Particle>,
    page: Page,
    editor_text: String,
    mouse: Vec2,
    mouse_raw: Vec2,
    mouse_down: bool,
    mouse_down_pos: Vec2,
    dragging: bool,
    x_axis: Axis,
    y_axis: Axis,
    x_axis_before: Axis,
    y_axis_before: Axis,
    settings_open: bool,
}

impl App {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let mut app = App {
            time: 0.0,
            renderer: Renderer::new(ctx)?,
            width: ctx.width(),
            height: ctx.height(),
            particles: Vec::new(),
            page: Page::Visualiser,
            editor_text: String::new(),
            mouse: Vec2::ZERO,
            mouse_raw: Vec2::ZERO,
            mouse_down: false,
            mouse_down_pos: Vec2::ZERO,
            dragging: false,
            x_axis: Axis::new(-1.0, 1.0),
            y_axis: Axis::new(-1.0, 1.0),
            x_axis_before: Axis::new(-1.0, 1.0),
            y_axis_before: Axis::new(-1.0, 1.0),
            settings_open: false,
        };
        app.correct_y_axis();
        Ok(app)
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
                                    Vec2::new(x.value(), y.value()),
                                ));
                            }
                        }
                    }
                }
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

    fn zoom(&mut self, zoom: f32, translate: bool) {
        // scale axis
        self.x_axis *= zoom;
        self.y_axis *= zoom;
        
        // translate to keep mouse at the same x and y
        if translate {
            self.x_axis -= -(self.mouse.x / zoom - self.mouse.x);
            self.y_axis -= -(self.mouse.y / zoom - self.mouse.y);
        }
    }

    fn correct_y_axis(&mut self) {
        let w = self.width as f32;
        let h = self.height as f32;
        let ratio = (w / h).min(h / w);
        self.y_axis = self.x_axis * ratio;
    }
}

#[derive(PartialEq, Eq)]
enum Page {
    Visualiser,
    Editor,
}

impl archie::event::EventHandler for App {
    fn update(&mut self, ctx: &archie::Context, dt: f32) {
        self.time += dt;

        self.width = ctx.width();
        self.height = ctx.height();

        if self.dragging && self.page == Page::Visualiser {
            let orig_mouse = map_pos(
                self.mouse_raw,
                vec2(self.width as f32, self.height as f32),
                self.x_axis_before,
                self.y_axis_before,
            );
            let dif = orig_mouse - self.mouse_down_pos;
            self.x_axis = self.x_axis_before - dif.x;
            self.y_axis = self.y_axis_before - dif.y;
        }
    }

    fn draw(
        &mut self,
        ctx: &archie::Context,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        if self.page == Page::Visualiser {
            let device = ctx.device();

            let particles = {
                let mut particles = [Particle::default(); 32];
                let slice = self.particles.as_slice();
                particles[..slice.len()].copy_from_slice(slice);
                let contents = bytemuck::cast_slice(&particles);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents,
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            };
            let constants = {
                let constants = ShaderConstants::new(
                    self.particles.len() as u32,
                    self.width,
                    self.height,
                    self.x_axis,
                    self.y_axis,
                );
                let contents = bytemuck::bytes_of(&constants);
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents,
                    usage: wgpu::BufferUsages::UNIFORM,
                })
            };

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: self.renderer.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: particles.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: constants.as_entire_binding(),
                    },
                ],
            });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });

                pass.set_pipeline(self.renderer.pipeline());
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
            }
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

        {
            let mut open = self.settings_open;
            egui::Window::new("Settings")
                .open(&mut open)
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

                    ui.separator();

                    if ui.button("Correct Ratio").clicked() {
                        self.correct_y_axis();
                    }
                });
            self.settings_open = open;
        }

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
                    let pos = self.mouse;
                    let arr = {
                        let mut particles = [Particle::default(); 32];
                        let slice = self.particles.as_slice();
                        particles[..slice.len()].copy_from_slice(slice);
                        particles
                    };
                    let len = self.particles.len();
                    let d = particle::dist(pos, &arr, len);
                    let v = particle::potential(pos, &arr, len);
                    let e = particle::force(pos, &arr, len);
                    ui.monospace(format!("distance (m): {}", d));
                    ui.monospace(format!("potential (J/C): {}", v));
                    ui.monospace(format!("force (N/C): {}", e));
                });
            }
        }
    }

    fn key_down(&mut self, key: VirtualKeyCode, modifiers: &ModifiersState) {
        match (*modifiers, key) {
            (ModifiersState::CTRL, VirtualKeyCode::Equals) => {
                // zoom in
                self.zoom(0.9, false);
            }
            (ModifiersState::CTRL, VirtualKeyCode::Minus) => {
                // zoom out
                self.zoom(1.1, false);
            }
            _ => (),
        }
    }

    fn mouse_up(&mut self, key: MouseButton) {
        if key == MouseButton::Left {
            self.mouse_down = false;
            self.dragging = false;
        }
    }

    fn mouse_down(&mut self, key: MouseButton) {
        if key == MouseButton::Left {
            self.mouse_down = true;
            self.mouse_down_pos = self.mouse;
        }
    }

    fn mouse_moved(&mut self, x: f64, y: f64) {
        let pos = Vec2::new(x as f32, y as f32);
        self.mouse_raw = pos;
        self.mouse = map_pos(
            pos,
            vec2(self.width as f32, self.height as f32),
            self.x_axis,
            self.y_axis,
        );
        let delta = ((self.mouse - self.mouse_down_pos)
            * vec2(self.width as f32, self.height as f32))
        .length();

        if self.mouse_down && delta > 6.0 {
            if !self.dragging {
                // start drag
                self.x_axis_before = self.x_axis;
                self.y_axis_before = self.y_axis;
            }
            self.dragging = true;
        }
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
        self.zoom(zoom, true);
    }
}

async fn run() {
    archie::log::init();

    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;

    let builder = archie::Context::builder()
        .title("Potential")
        .width(WIDTH)
        .height(HEIGHT)
        .fullscreen(cfg!(target_arch = "wasm32")); // fullscreen for wasm

    match builder.build(None).await {
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
