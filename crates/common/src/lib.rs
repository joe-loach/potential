#![cfg_attr(target_arch = "spirv", no_std)]

mod axis;

pub use axis::Axis;

use glam::Vec2;

pub fn map_pos(pos: Vec2, res: Vec2, x_axis: Axis, y_axis: Axis) -> Vec2 {
    // [0, a]
    // [0, 1]       (/a)
    // [0, c-b]     (*(c-b))
    // [b, c]       + b
    fn map(x: f32, a: f32, b: f32, c: f32) -> f32 {
        (x / a) * (c - b) + b
    }
    let (x, y) = (pos.x, pos.y);
    let x = map(x, res.x, x_axis.min, x_axis.max);
    let y = map(y, res.y, y_axis.max, y_axis.min);
    Vec2::new(x, y)
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShaderConstants {
    __pad__: u32,
    pub empty: u32,
    pub width: u32,
    pub height: u32,
    pub x_axis: Axis,
    pub y_axis: Axis,
}

impl ShaderConstants {
    pub fn new(empty: u32, width: u32, height: u32, x_axis: Axis, y_axis: Axis) -> Self {
        Self {
            empty,
            width,
            height,
            x_axis,
            y_axis,
            __pad__: 0,
        }
    }
}

#[cfg(not(target_arch = "spirv"))]
mod need_std {
    use super::*;

    use std::{collections::HashMap, path::PathBuf};
    pub use toml;

    use serde_derive::{Deserialize, Serialize};

    #[derive(Default, Serialize, Deserialize)]
    pub struct Config {
        pub shaders: HashMap<String, ShaderInfo>,
    }

    #[derive(Default, Serialize, Deserialize)]
    pub struct ShaderInfo {
        pub entries: Vec<String>,
        pub module: PathBuf,
    }

    impl ShaderConstants {
        pub fn as_bytes(&self) -> &[u8] {
            bytemuck::bytes_of(self)
        }
    }

    unsafe impl bytemuck::Pod for ShaderConstants {}
    unsafe impl bytemuck::Zeroable for ShaderConstants {}
}
#[cfg(not(target_arch = "spirv"))]
pub use need_std::*;
