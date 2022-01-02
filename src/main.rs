extern crate ultraviolet as uv;

mod axis;

pub use axis::*;

use std::{collections::HashMap, rc::Rc};

use potential::{
    poml::{parser::ast, Registry},
    Context, Field, Force, Index, Object, Potential, Shape, Store,
};

#[derive(Default)]
pub struct Program {
    map: HashMap<String, Index<Shape>>,
    shapes: Rc<Store<Shape>>,
    objects: Vec<Object>,
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {
    width: u32,
    height: u32,
    program: Program,
    registry: Registry,
    page: Page,
    editor_text: String,
    recompiled: bool,
    mouse: uv::Vec2,
    x_axis: Axis,
    y_axis: Axis,
    settings_open: bool,
}

impl App {
    pub fn new(_ctx: &mut Context) -> Self {
        let registry = Registry::default().register("circle", potential::shapes::circle);

        App {
            width: WIDTH,
            height: HEIGHT,
            program: Program::default(),
            registry,
            page: Page::Visualiser,
            editor_text: String::new(),
            recompiled: false,
            mouse: uv::Vec2::zero(),
            x_axis: Axis::new(-1.0, 1.0),
            y_axis: Axis::new(-1.0, 1.0),
            settings_open: false,
        }
    }

    pub fn compile(&mut self) {
        let parse = potential::poml::compile(&self.editor_text, &self.registry);

        match parse {
            Ok(parse) => {
                // clear out the old information
                self.program.map.clear();
                self.program.objects.clear();
                // SAFETY:
                // * all the indexes in the map have been cleared
                // * all the objects have been cleared so there are no other references
                unsafe {
                    let shapes = Rc::get_mut(&mut self.program.shapes).unwrap();
                    shapes.clear();
                }

                let root = parse.root();
                // Collect all shapes and their labels
                {
                    let shapes =
                        Rc::get_mut(&mut self.program.shapes).expect("no other references to cell");
                    for s in root.stmts() {
                        if let ast::StmtKind::Shape(shape) = s.kind() {
                            // get the name and label of the shape
                            let label = shape.label().text().unwrap();
                            let name = shape.name().map(|n| n.text()).unwrap();
                            // collect the parameters passed to the shape
                            let params = s.params().unwrap().values().map(|v| v.value()).collect();
                            // create the shape
                            let sdf = self.registry.call(&name, params).unwrap();
                            // add it to the list of shapes
                            let index = shapes.push(sdf);
                            // save the transformation of label to index for later
                            self.program.map.insert(label, index);
                        }
                    }
                }
                // Collect all objects and their values
                for s in root.stmts() {
                    if let ast::StmtKind::Object(_) = s.kind() {
                        // get all of the parameters for the object
                        let mut params = s.params().unwrap();
                        let value = params.next_value().unwrap();
                        let x = params.next_value().unwrap();
                        let y = params.next_value().unwrap();
                        let label = params.next_name().unwrap();
                        // use the transformation map to get the index for the shape
                        if let Some(index) = self.program.map.get(&label.text()) {
                            let shape = self.program.shapes.get(index);
                            // create the object
                            let object = Object::new(
                                value.value(),
                                uv::Vec2::new(x.value(), y.value()),
                                *shape,
                            );
                            // add the object to the list
                            self.program.objects.push(object);
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
    pub fn dist(&self, pos: uv::Vec2) -> f32 {
        let mut d = f32::INFINITY;
        for obj in self.program.objects.iter() {
            d = d.min(*obj.at(pos));
        }
        d
    }

    pub fn potential(&self, pos: uv::Vec2) -> Potential {
        self.program.objects.as_slice().at(pos)
    }

    pub fn force(&self, pos: uv::Vec2) -> Force {
        self.program.objects.as_slice().at(pos)
    }

    fn map_pos(&self, pos: uv::Vec2) -> uv::Vec2 {
        // [0, a]
        // [0, 1]       (/a)
        // [0, c-b]     (*(c-b))
        // [b, c]       + b
        fn map(x: f32, a: f32, b: f32, c: f32) -> f32 {
            (x / a) * (c - b) + b
        }
        let uv::Vec2 { x, y } = pos;
        let x = map(x, self.width as f32, self.x_axis.min(), self.x_axis.max());
        let y = map(y, self.height as f32, self.y_axis.max(), self.y_axis.min());
        uv::Vec2::new(x, y)
    }
}

#[derive(PartialEq, Eq)]
enum Page {
    Visualiser,
    Editor,
}

impl potential::EventHandler for App {
    fn update(&mut self, ctx: &Context, _dt: f32) {
        self.width = ctx.width();
        self.height = ctx.height();

        if self.recompiled {
            println!("Recompiled");
            self.recompiled = false;
        }
    }

    fn draw(&mut self, _encoder: &mut wgpu::CommandEncoder, _target: &wgpu::TextureView) {
        if self.page == Page::Visualiser {
            // draw
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
                                for (i, obj) in self.program.objects.iter().enumerate() {
                                    egui::CollapsingHeader::new(i.to_string())
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            ui.monospace(format!("shape: {:?}", obj.shape));
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
                    let v = self.potential(self.mouse);
                    let e = self.force(self.mouse);
                    ui.monospace(format!("distance (m): {}", self.dist(self.mouse)));
                    ui.monospace(format!("potential (J/C): {{{}, {}}}", v.x, v.y));
                    ui.monospace(format!("force (N/C): {{{}, {}}}", e.x, e.y));
                });
            }
        }
    }

    fn mouse_moved(&mut self, x: f64, y: f64) {
        let pos = uv::Vec2::new(x as f32, y as f32);
        self.mouse = self.map_pos(pos);
    }

    fn wheel_moved(&mut self, _dx: f32, dy: f32) {
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
    let builder = Context::builder()
        .title("Potential")
        .width(WIDTH)
        .height(HEIGHT)
        .fullscreen(cfg!(target_arch = "wasm32")); // fullscreen for wasm

    match builder.build().await {
        Ok((event_loop, mut ctx)) => {
            let app = App::new(&mut ctx);

            potential::event::run(ctx, event_loop, app)
        }
        Err(e) => {
            eprintln!("failed to build potential context: {:?}", e);
        }
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}
