extern crate ultraviolet as uv;

use std::{collections::HashMap, rc::Rc};

use potential::{
    poml::{parser::ast, Registry},
    Context, Index, Object, Sdf, Store,
};

#[derive(Default)]
pub struct Program {
    map: HashMap<String, Index<Box<dyn Sdf>>>,
    shapes: Rc<Store<Box<dyn Sdf>>>,
    objects: Vec<Object>,
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {
    program: Program,
    registry: Registry,
    page: Page,
    editor_text: String,
}

impl App {
    pub fn new(_ctx: &mut Context) -> Self {
        let registry = Registry::default().register("circle", potential::sdf::Circle::new);

        App {
            program: Program::default(),
            registry,
            page: Page::Visualiser,
            editor_text: String::new(),
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
                for s in root.stmts() {
                    let shapes =
                        Rc::get_mut(&mut self.program.shapes).expect("no other references to cell");
                    match s.kind() {
                        ast::StmtKind::Shape(shape) => {
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
                        _ => (),
                    }
                }
                // Collect all objects and their values
                for s in root.stmts() {
                    match s.kind() {
                        ast::StmtKind::Object(_) => {
                            // get all of the parameters for the object
                            let mut params = s.params().unwrap();
                            let value = params.next_value().unwrap();
                            let x = params.next_value().unwrap();
                            let y = params.next_value().unwrap();
                            let label = params.next_name().unwrap();
                            // use the transformation map to get the index for the shape
                            if let Some(&index) = self.program.map.get(&label.text()) {
                                // create the object with the cloned store
                                let object = Object::new(
                                    value.value(),
                                    uv::Vec2::new(x.value(), y.value()),
                                    index,
                                    Rc::clone(&self.program.shapes),
                                );
                                // add the object to the list
                                self.program.objects.push(object);
                            }
                        }
                        _ => (),
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
}

#[derive(PartialEq, Eq)]
enum Page {
    Visualiser,
    Editor,
}

impl potential::EventHandler for App {
    fn update(&mut self) {}

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
            })
        });

        if self.page == Page::Editor {
            egui::TopBottomPanel::bottom("editor_bottom").show(ctx, |ui| {
                let layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
                ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                    if ui.button("Compile").clicked() {
                        self.compile();
                    }
                })
            });

            egui::CentralPanel::default().show(ctx, |ui| {
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
