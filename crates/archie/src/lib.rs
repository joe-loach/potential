mod context;
mod helper;

pub mod event;

pub use context::*;

pub use egui;
pub use wgpu;
pub use winit;

use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn block_on(fut: impl Future<Output = ()> + 'static) {
    pollster::block_on(fut);
}

#[cfg(target_arch = "wasm32")]
pub fn block_on(fut: impl Future<Output = ()> + 'static) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(fut);
}
