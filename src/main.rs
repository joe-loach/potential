const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {
    selected_anchor: Anchor,
}

#[derive(PartialEq, Eq)]
enum Anchor {
    Visualiser,
    Editor,
}

impl potential::EventHandler for App {
    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn ui(&mut self, ctx: &egui::CtxRef) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.label("Potential");
                ui.separator();

                if ui
                    .selectable_label(self.selected_anchor == Anchor::Visualiser, "Visualiser")
                    .clicked()
                {
                    self.selected_anchor = Anchor::Visualiser;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#visualiser");
                    }
                }
                if ui
                    .selectable_label(self.selected_anchor == Anchor::Editor, "Editor")
                    .clicked()
                {
                    self.selected_anchor = Anchor::Editor;
                    if cfg!(target_arch = "wasm32") {
                        ui.output().open_url("#editor");
                    }
                }
            })
        });
    }
}

impl Default for App {
    fn default() -> Self {
        App {
            selected_anchor: Anchor::Visualiser,
        }
    }
}

async fn run() {
    let builder = potential::Context::builder()
        .title("Potential")
        .width(WIDTH)
        .height(HEIGHT)
        .fullscreen(cfg!(target_arch = "wasm32")); // fullscreen for wasm

    match builder.build().await {
        Ok((event_loop, ctx)) => {
            let app = App::default();

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
