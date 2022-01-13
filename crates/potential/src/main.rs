mod renderer;

use anyhow::Result;
use archie::{
    egui, wgpu,
    winit::event::{ModifiersState, MouseButton, VirtualKeyCode},
};
use egui_nodes::{NodeArgs, NodeConstructor};
use glam::{vec2, Vec2};
use particle::Particle;

use common::*;
use renderer::Renderer;

// TODO: write information to a egui texture
// https://github.com/emilk/egui/blob/master/egui_glium/examples/native_texture.rs

struct App {
    time: f32,
    renderer: Renderer,
    nodes: egui_nodes::Context,
    width: u32,
    height: u32,
    particles: Vec<Particle>,
    page: Page,
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
            nodes: egui_nodes::Context::default(),
            width: ctx.width(),
            height: ctx.height(),
            particles: Vec::new(),
            page: Page::Potential,
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
    Potential,
    Force,
}

impl archie::event::EventHandler for App {
    fn update(&mut self, ctx: &archie::Context, dt: f32) {
        self.time += dt;

        self.width = ctx.width();
        self.height = ctx.height();

        if self.dragging && self.page == Page::Potential {
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
        let device = ctx.device();

        self.renderer.update(
            device,
            &self.particles,
            self.width,
            self.height,
            self.x_axis,
            self.y_axis,
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.renderer.bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.renderer.particles(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.renderer.constants(),
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

    fn ui(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.label("Field Visualiser");
                ui.separator();

                if ui
                    .selectable_label(self.page == Page::Potential, "Potential")
                    .clicked()
                {
                    self.page = Page::Potential;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#potential");
                    }
                }
                if ui
                    .selectable_label(self.page == Page::Force, "Force")
                    .clicked()
                {
                    self.page = Page::Force;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#force");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.button("🔧").on_hover_text("Settings").clicked() {
                        self.settings_open = !self.settings_open;
                    }
                })
            })
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Particles");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui
                        .button("➖")
                        .on_hover_text("Delete selected node")
                        .clicked()
                    {
                        if let Some(idx) = self.nodes.get_selected_nodes().pop() {
                            self.particles.remove(idx);
                        }
                    }
                    if ui.button("➕").on_hover_text("Add a new Node").clicked() {
                        self.particles.push(Default::default());
                    }
                });
            });

            {
                use egui_nodes::ColorStyle::*;
                let theme = &ctx.style().visuals;
                self.nodes.style.colors[NodeBackground as usize] = theme.widgets.active.bg_fill;
                self.nodes.style.colors[NodeBackgroundHovered as usize] =
                    theme.widgets.hovered.bg_fill;
                self.nodes.style.colors[TitleBar as usize] = theme.faint_bg_color;
                self.nodes.style.colors[TitleBarHovered as usize] = theme.extreme_bg_color;
                self.nodes.style.colors[TitleBarSelected as usize] = theme.selection.bg_fill;
            }

            let nodes = self.particles.iter_mut().enumerate().map(|(i, p)| {
                NodeConstructor::new(
                    i,
                    NodeArgs {
                        corner_rounding: Some(1.0),
                        ..Default::default()
                    },
                )
                .with_title(move |ui| ui.label("Particle"))
                .with_static_attribute(0, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Value");
                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                            ui.add(egui::DragValue::new(&mut p.value))
                        })
                    })
                    .response
                })
                .with_static_attribute(1, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Radius");
                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                            ui.add(
                                egui::DragValue::new(&mut p.radius)
                                    .clamp_range(f32::EPSILON..=f32::INFINITY),
                            )
                        })
                    })
                    .response
                })
                .with_static_attribute(2, |ui| {
                    let x = ui
                        .horizontal(|ui| {
                            ui.label("X");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(egui::DragValue::new(&mut p.pos.x))
                            })
                        })
                        .response;
                    let y = ui
                        .horizontal(|ui| {
                            ui.label("Y");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(egui::DragValue::new(&mut p.pos.y))
                            })
                        })
                        .response;
                    x.union(y)
                })
            });

            self.nodes.show(nodes, std::iter::empty(), ui);
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
        if self.page == Page::Force {
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
