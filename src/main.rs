use potential::Context;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {
    page: Page,
}

impl App {
    pub fn new(_ctx: &mut Context) -> Self {
        App {
            page: Page::Visualiser,
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
        // draw potentials when on the correct page
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
