mod nodes;
mod renderer;
mod scientific;

use anyhow::Result;
use archie::{
    egui, wgpu,
    winit::event::{ModifiersState, MouseButton, VirtualKeyCode},
};
use glam::*;

use nodes::*;
use scientific::Sci;
use renderer::Renderer;

use particle::Particle;
use common::*;

struct App {
    time: f32,
    renderer: Renderer,
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
    texture: wgpu::Texture,
    texture_id: egui::TextureId,
    texture_size: UVec2,
    texture_pos: Vec2,
    on_image: bool,
    help_open: bool,
    color_open: bool,
    colors: [[f32; 4]; 2],
}

impl App {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: ctx.width(),
                height: ctx.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let texture_id = ctx.egui_register_texture(&texture, wgpu::FilterMode::Linear);

        let mut app = App {
            time: 0.0,
            renderer: Renderer::new(ctx)?,
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
            texture,
            texture_id,
            texture_size: vec2(100.0, 100.0),
            texture_pos: vec2(0.0, 0.0),
            on_image: false,
            help_open: false,
            color_open: false,
            colors: [[0.0, 0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]],
        };
        app.correct_y_axis();
        Ok(app)
    }

    fn place_image(&mut self, ctx: &egui::CtxRef, ui: &mut egui::Ui, rect: egui::Rect) {
        let size = rect.size();
        self.texture_size = vec2(size.x, size.y);
        let image = ui.put(
            rect,
            egui::Image::new(self.texture_id, size).sense(egui::Sense::focusable_noninteractive()),
        );
        let rect = image.rect;
        self.texture_pos = {
            let top_left = rect.min;
            vec2(top_left.x, top_left.y)
        };
        self.on_image = ui.rect_contains_pointer(rect) && !ctx.wants_pointer_input();
    }

    fn zoom(&mut self, zoom: f32, translate: bool) {
        if !self.on_image {
            return;
        }
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
        let w = self.texture_size.x;
        let h = self.texture_size.y;
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

        if self.dragging && self.on_image {
            let orig_mouse = map_pos(
                self.mouse_raw - self.texture_pos,
                self.texture_size,
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
        ctx: &mut archie::Context,
        encoder: &mut wgpu::CommandEncoder,
        _: &wgpu::TextureView,
    ) {
        let width = (self.texture_size.x * self.scaling) as u32;
        let height = (self.texture_size.y * self.scaling) as u32;

        // resize texture to make computation easier
        self.texture = ctx.device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let bind_group = {
            let device = ctx.device();
            // update the renderer with latest information
            self.renderer.update(
                device,
                &self.particles,
                match self.page {
                    Page::Potential => Field::Potential,
                    Page::Force => Field::Force,
                },
                glam::uvec2(width, height),
                self.x_axis,
                self.y_axis,
                self.colors,
            );
            // make a new bind group
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: self.renderer.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.renderer.constants(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.renderer.particles(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.renderer.colors(),
                    },
                ],
            })
        };
        // run shader
        {
            let view = &self
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view,
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
        // update the egui texture
        ctx.egui_update_texture(&self.texture, wgpu::FilterMode::Linear, self.texture_id);
    }

    fn ui(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.heading("Field Visualiser");
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
                    if ui
                        .button("Help")
                        .on_hover_text("Opens help dialogue")
                        .clicked()
                    {
                        self.help_open = !self.help_open;
                    }
                    if ui
                        .button("✏")
                        .on_hover_text("Opens color dialogue")
                        .clicked()
                    {
                        self.color_open = !self.color_open;
                    }
                    if ui.button("Correct Ratio").clicked() {
                        self.correct_y_axis();
                    }
                })
            })
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Particles");
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    if ui.button("➕").on_hover_text("Add a new Node").clicked() {
                        self.particles.push(Default::default());
                    }
                });
            });

            let remove_idx = std::rc::Rc::new(std::sync::Mutex::new(None));
            let nodes = self.particles.iter_mut().enumerate().map(|(i, p)| {
                let rm_idx = std::rc::Rc::clone(&remove_idx);
                Node::new()
                    .with_header(move |ui| {
                        ui.horizontal(|ui| {
                            ui.strong("Particle");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                if ui
                                    .button("➖")
                                    .on_hover_text("Delete selected node")
                                    .clicked()
                                {
                                    rm_idx.lock().unwrap().replace(i);
                                }
                            });
                        })
                        .response
                    })
                    .with_body(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Value");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(egui::DragValue::new(&mut p.value))
                            })
                        });
                        ui.horizontal(|ui| {
                            ui.label("Radius");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(
                                    egui::DragValue::new(&mut p.radius)
                                        .clamp_range(f32::EPSILON..=f32::INFINITY),
                                )
                            })
                        });
                        ui.horizontal(|ui| {
                            ui.label("X");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(egui::DragValue::new(&mut p.pos.x))
                            })
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y");
                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.add(egui::DragValue::new(&mut p.pos.y))
                            })
                        })
                        .response
                    })
            });

            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    NodePanel::ui(nodes, ui);
                });

            let idx = remove_idx.lock().unwrap();
            if let Some(&idx) = idx.as_ref() {
                self.particles.remove(idx);
            }
        });

        let text_color = ctx.style().visuals.noninteractive().text_color();
        let frame = egui::Frame::none().fill(ctx.style().visuals.window_fill());
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            let rect = ui.max_rect();
            let size = ui.available_size();

            const SHRINK_FACTOR: f32 = 75.0;
            let img_rect = rect.shrink(SHRINK_FACTOR);
            let img_size = img_rect.size();

            let (_, painter) = ui.allocate_painter(size, egui::Sense::hover());
            painter.text(
                egui::pos2(img_rect.left(), img_rect.top() - 6.0),
                egui::Align2::LEFT_BOTTOM,
                Sci(self.x_axis.min),
                egui::TextStyle::Monospace,
                text_color,
            );
            painter.text(
                egui::pos2(img_rect.right(), img_rect.top() - 6.0),
                egui::Align2::RIGHT_BOTTOM,
                Sci(self.x_axis.max),
                egui::TextStyle::Monospace,
                text_color,
            );
            painter.text(
                egui::pos2(rect.left() + 2.0, img_rect.top()),
                egui::Align2::LEFT_TOP,
                Sci(self.y_axis.max),
                egui::TextStyle::Monospace,
                text_color,
            );
            painter.text(
                egui::pos2(rect.left() + 2.0, img_rect.bottom()),
                egui::Align2::LEFT_BOTTOM,
                Sci(self.y_axis.min),
                egui::TextStyle::Monospace,
                text_color,
            );
            let stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            for i in 0..21 {
                let x = img_rect.left() + (img_size.x / 20.0) * i as f32;
                painter.line_segment(
                    [
                        egui::pos2(x, img_rect.top()),
                        egui::pos2(x, img_rect.top() - 5.0),
                    ],
                    stroke,
                );
            }
            for i in 0..21 {
                let y = img_rect.top() + (img_size.y / 20.0) * i as f32;
                painter.line_segment(
                    [
                        egui::pos2(img_rect.left(), y),
                        egui::pos2(img_rect.left() - 5.0, y),
                    ],
                    stroke,
                );
            }

            self.place_image(ctx, ui, img_rect);

            ui.horizontal(|ui| {
                ui.centered_and_justified(|ui| {
                    let pos = self.mouse;
                    let arr = {
                        let mut particles = [Particle::default(); 32];
                        let slice = self.particles.as_slice();
                        particles[..slice.len().min(32)].copy_from_slice(slice);
                        particles
                    };
                    let len = self.particles.len();
                    let d = particle::dist(pos, &arr, len).unwrap_or(0.0);
                    let v = particle::potential(pos, &arr, len);
                    let e = particle::force(pos, &arr, len);
                    ui.monospace(format!("pos = {}, {}", Sci(pos.x), Sci(pos.y)));
                    ui.monospace(format!("d = {} m", Sci(d)));
                    ui.monospace(format!("V = {} J/C", Sci(v)));
                    ui.monospace(format!("F = {} N/C", Sci(e)));
                });
            });
        });

        {
            let mut open = self.help_open;
            egui::Window::new("Help").open(&mut open).show(ctx, |ui| {
                egui::CollapsingHeader::new("Usage")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(
                            "Add particles in the right side bar by clicking the ➕.\n\
                            Edit the values of each particle to suit your needs.\n\
                            After any particle is added, the visualiser should show it.\n\
                            Move around the field by dragging the mouse with LMB held.\n\
                            Zoom into a point using the scroll wheel.\n\
                            Zoom into the center using Ctrl + (- =)\n\
                            ",
                        );
                    });
            });
            self.help_open = open;
        }

        {
            let mut open = self.color_open;
            let a = &mut self.colors[0][1..].try_into().unwrap();
            let b = &mut self.colors[1][1..].try_into().unwrap();
            egui::Window::new("Colors").open(&mut open).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.color_edit_button_rgb(a);
                    let b = self.colors[1][0];
                    ui.add(
                        egui::DragValue::new(&mut self.colors[0][0])
                            .min_decimals(2)
                            .clamp_range(-f32::INFINITY..=b),
                    );
                });
                ui.horizontal(|ui| {
                    ui.color_edit_button_rgb(b);
                    let a = self.colors[0][0];
                    ui.add(
                        egui::DragValue::new(&mut self.colors[1][0])
                            .min_decimals(2)
                            .clamp_range(a..=f32::INFINITY),
                    );
                });
            });
            self.colors[0][1..].copy_from_slice(a);
            self.colors[1][1..].copy_from_slice(b);
            self.color_open = open;
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
            pos - self.texture_pos,
            self.texture_size,
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
        // zoom into a point
        const ZOOM_INTENSITY: f32 = -0.1;
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
