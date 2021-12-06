const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

struct App {}

impl potential::EventHandler for App {
    type Event = ();

    fn update(&mut self) {}
    fn draw(&mut self) {}

    fn ui(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame) {
        egui::Window::new("Test").show(ctx, |ui| {
            if ui.button("click").clicked() {
                // do something
            }
        });
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
            let app = App {};

            potential::event::run(ctx, event_loop, app)
        }
        Err(e) => {
            eprintln!("error: {:?}", e);
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
