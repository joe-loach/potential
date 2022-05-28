mod appearance;

use anyhow::Result;
use archie::wgpu;
use archie_egui::egui;

use potential::{graph::Figure, Particle};

#[derive(PartialEq)]
enum Tab {
    Graph,
    Editor,
}

struct WindowsOpen {
    settings: bool,
    timings: bool,
}

pub struct App {
    gui: archie_egui::Egui,
    tab: Tab,
    figure: Figure,
    particles: Vec<Particle>,
    open: WindowsOpen,
}

impl App {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let gui = archie_egui::Egui::new(&*ctx);
        gui.context().set_style(appearance::style());
        gui.context().set_fonts(appearance::fonts());

        let app = App {
            gui,
            tab: Tab::Graph,
            figure: Figure::new(100.0, 100.0),
            particles: vec![
                Particle::new(1.0, 2.0, glam::Vec2::new(1.0, 1.0)),
                Particle::new(1.0, 2.0, glam::Vec2::new(1.0, 1.0)),
            ],
            open: WindowsOpen {
                settings: false,
                timings: false,
            },
        };
        Ok(app)
    }
}

use egui::{
    style::Margin, CentralPanel, DragValue, Frame, Layout, RichText, TopBottomPanel, Window,
};

impl archie::event::EventHandler for App {
    fn update(&mut self, ctx: &archie::Context) {
        let timer = ctx.timer();
        let avg = timer.average().as_secs_f64();

        self.gui.update(ctx, |gui| {
            title_bar(gui, &mut self.tab, &mut self.open);

            Window::new("Timings")
                .open(&mut self.open.timings)
                .resizable(false)
                .frame(Frame::window(&gui.style()).multiply_with_opacity(0.5))
                .show(gui, |ui| {
                    let fps = 1.0 / avg;
                    let spf = 1000.0 * avg;

                    ui.monospace(format!("FPS: {:.1}", fps));
                    ui.monospace(format!("SPF: {:.3} ms", spf));

                    use egui::plot::*;

                    let timings = timer.timings();
                    let values = timings.iter().enumerate().map(|(i, dur)| {
                        Value::new(i as f64 / timings.len() as f64, dur.as_secs_f64())
                    });
                    let line = Line::new(Values::from_values_iter(values));
                    Plot::new("Timings Plot")
                        .width(300.0)
                        .height(300.0)
                        .allow_drag(false)
                        .allow_zoom(false)
                        .show(ui, |ui| ui.line(line));
                });

            match self.tab {
                Tab::Graph => {
                    Window::new("Settings")
                        .open(&mut self.open.settings)
                        .resizable(false)
                        .frame(Frame::window(&gui.style()).multiply_with_opacity(0.5))
                        .show(gui, |ui| {
                            if ui.button("Timings").clicked() {
                                self.open.timings = true;
                            }
                            figure_ui(ui, &mut self.figure);
                        });
                }
                Tab::Editor => {
                    CentralPanel::default()
                        .frame(Frame::none().margin(Margin::same(4.0)))
                        .show(gui, |ui| {
                            for (i, p) in self.particles.iter_mut().enumerate() {
                                particle_window(gui, ui, p, i);
                            }
                        });
                }
            }
        });
    }

    fn draw(
        &mut self,
        ctx: &mut archie::Context,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        self.gui.draw(ctx, encoder, view);
    }

    fn raw_event(&mut self, _ctx: &mut archie::Context, event: &archie::winit::event::Event<()>) {
        self.gui.handle_event(event);
    }
}

fn particle_window(gui: &egui::Context, ui: &egui::Ui, p: &mut Particle, i: usize) {
    Window::new("Particle")
        .title_bar(false)
        .resizable(false)
        .id(ui.id().with(format!("Particle_{}", i)))
        .show(gui, |ui| {
            ui.heading("Particle");
            ui.horizontal(|ui| {
                ui.label("Value ");
                ui.add(DragValue::new(&mut p.value).suffix(" v"));
            });
            ui.horizontal(|ui| {
                ui.label("Radius");
                ui.add(DragValue::new(&mut p.radius).suffix(" m"));
            });
            ui.label("Pos");
            ui.horizontal(|ui| {
                ui.add(DragValue::new(&mut p.pos.x).prefix("x: ").suffix(" m"));
                ui.add(DragValue::new(&mut p.pos.y).prefix("y: ").suffix(" m"));
            });
        });
}

fn title_bar(gui: &egui::Context, tab: &mut Tab, window_open: &mut WindowsOpen) {
    TopBottomPanel::top("Title Bar")
        .frame(
            Frame::window(&gui.style())
                .margin(Margin::same(2.0))
                .multiply_with_opacity(0.5),
        )
        .height_range(0.0..=22.0)
        .show(gui, |ui| {
            ui.horizontal_top(|ui| {
                ui.selectable_value(tab, Tab::Graph, RichText::new("Graph").heading());
                ui.selectable_value(tab, Tab::Editor, RichText::new("Editor").heading());

                ui.with_layout(Layout::right_to_left(), |ui| {
                    if ui.button("⛭").clicked() {
                        window_open.settings = true;
                    }
                });
            })
        });
}

fn figure_ui(ui: &mut egui::Ui, f: &mut Figure) {
    let Figure {
        x_min,
        x_max,
        y_min,
        y_max,
        ..
    } = f;

    ui.heading("Figure");
    ui.horizontal(|ui| {
        ui.add(
            DragValue::new(x_min)
                .clamp_range(f32::NEG_INFINITY..=*x_max)
                .fixed_decimals(2),
        );
        ui.monospace("≤ X ≤");
        ui.add(
            DragValue::new(x_max)
                .clamp_range(*x_min..=f32::INFINITY)
                .fixed_decimals(2),
        );
    });
    ui.horizontal(|ui| {
        ui.add(
            DragValue::new(y_min)
                .clamp_range(f32::NEG_INFINITY..=*y_max)
                .fixed_decimals(2),
        );
        ui.monospace("≤ Y ≤");
        ui.add(
            DragValue::new(y_max)
                .clamp_range(*y_min..=f32::INFINITY)
                .fixed_decimals(2),
        );
    });
}
