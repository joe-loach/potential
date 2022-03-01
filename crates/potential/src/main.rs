mod app;

use app::App;

async fn run() {
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
    archie::log::init();

    archie::block_on(run());
}
