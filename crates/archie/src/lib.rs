mod context;
mod helper;

pub mod event;

pub use context::*;

pub use egui;
pub use wgpu;
pub use winit;

use std::future::Future;

pub mod log {
    pub fn init() {
        cfg_if::cfg_if! {
            if #[cfg(web)] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                let _ = wasm_logger::init(wasm_logger::Config::default());
            } else {
                let _ = env_logger::try_init();
            }
        }
    }
}

pub fn block_on(fut: impl Future<Output = ()> + 'static) {
    cfg_if::cfg_if! {
        if #[cfg(web)] {
            wasm_bindgen_futures::spawn_local(fut);
        } else {
            pollster::block_on(fut);
        }
    }
}
