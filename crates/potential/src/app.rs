mod appearance;

use anyhow::Result;
use archie::wgpu;

#[derive(PartialEq)]
enum Tab {
    Graph,
    Editor,
}

pub struct App {
    gui: archie_egui::Egui,
    tab: Tab,
}

impl App {
    pub fn new(ctx: &mut archie::Context) -> Result<Self> {
        let gui = archie_egui::Egui::new(&*ctx);
        gui.context().set_style(appearance::style());
        gui.context().set_fonts(appearance::fonts());

        let app = App {
            gui,
            tab: Tab::Graph,
        };
        Ok(app)
    }
}

use egui::{Frame, RichText, TopBottomPanel, Window, CentralPanel};
use potential::graph::Figure;

impl archie::event::EventHandler for App {
    fn update(&mut self, ctx: &archie::Context) {
        let dt = ctx.timer().delta().as_secs_f64();
        self.gui.update(ctx, |gui| {
            TopBottomPanel::top("Title Bar")
                .frame(
                    Frame::window(&gui.style())
                        .margin(egui::style::Margin {
                            left: 4.0,
                            right: 4.0,
                            top: 4.0,
                            bottom: 0.0,
                        })
                        .multiply_with_opacity(0.5),
                )
                .height_range(22.0..=22.0)
                .show(gui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.selectable_value(
                            &mut self.tab,
                            Tab::Graph,
                            RichText::new("Graph").heading(),
                        );
                        ui.selectable_value(
                            &mut self.tab,
                            Tab::Editor,
                            RichText::new("Editor").heading(),
                        );

                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                            let dt_ms = 1000.0 * dt;
                            let fps = 1.0 / dt;
                            ui.label(format!("{:.3} ms/frame ({:.1} FPS)", dt_ms, fps));
                        });
                    })
                });

            match self.tab {
                Tab::Graph => {
                    Window::new("Settings")
                        .frame(Frame::window(&gui.style()).multiply_with_opacity(0.5))
                        .show(gui, |ui| figure_ui(ui, &mut Figure::new(100.0, 100.0)));
                }
                Tab::Editor => {
                    CentralPanel::default().show(gui, |ui| {
                        ui.heading("TODO");
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
            egui::DragValue::new(x_min)
                .clamp_range(-f32::INFINITY..=*x_max)
                .fixed_decimals(2),
        );
        ui.monospace("≤ X ≤");
        ui.add(
            egui::DragValue::new(x_max)
                .clamp_range(*x_min..=f32::INFINITY)
                .fixed_decimals(2),
        );
    });
    ui.horizontal(|ui| {
        ui.add(
            egui::DragValue::new(y_min)
                .clamp_range(-f32::INFINITY..=*y_max)
                .fixed_decimals(2),
        );
        ui.monospace("≤ Y ≤");
        ui.add(
            egui::DragValue::new(y_max)
                .clamp_range(*y_min..=f32::INFINITY)
                .fixed_decimals(2),
        );
    });
}
