extern crate log as _log;

mod context;
mod timer;

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
                let query_string = web_sys::window().unwrap().location().search().unwrap();
                let level = super::helper::parse_url_query_string(&query_string, "RUST_LOG")
                    .map(|x| x.parse().ok())
                    .flatten()
                    .unwrap_or(_log::Level::Error);
                let _ = wasm_logger::init(wasm_logger::Config::new(level));
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
