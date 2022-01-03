mod context;
mod helper;

pub mod event;

pub use context::*;

pub use egui;
pub use wgpu;
pub use winit;

use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn log(level: log::Level) {
    use simplelog::*;
    let level = match level {
        Level::Error => LevelFilter::Error,
        Level::Warn => LevelFilter::Warn,
        Level::Info => LevelFilter::Info,
        Level::Debug => LevelFilter::Debug,
        Level::Trace => LevelFilter::Trace,
    };
    let _ = SimpleLogger::init(level, Config::default());
}

#[cfg(target_arch = "wasm32")]
pub fn log(level: log::Level) {
    let _ = console_log::init_with_level(level);
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn block_on(fut: impl Future<Output = ()> + 'static) {
    pollster::block_on(fut);
}

#[cfg(target_arch = "wasm32")]
pub fn block_on(fut: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(fut);
}
