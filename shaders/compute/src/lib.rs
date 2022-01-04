#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]

extern crate spirv_std;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[cfg(target_arch = "spirv")]
use crate::spirv_std::num_traits::Float;

use spirv_std::glam::*;

use common::*;
use particle::*;

#[spirv(fragment)]
pub fn frag(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] buffer: &[Particle],
    output: &mut Vec4,
) {
    let pos = pos.xy();
    let res = vec2(constants.width as f32, constants.height as f32);
    let pos = map_pos(pos, res, constants.x_axis, constants.y_axis);
    let v = match constants.empty != 0 {
        false => {
            let v = potential(pos, buffer);
            v.abs().clamp(0.0, 1.0)
        }
        true => 0.0,
    };
    *output = vec4(v, v, v, 1.0);
}

#[spirv(vertex)]
pub fn vert(#[spirv(vertex_index)] idx: i32, #[spirv(position)] coord: &mut Vec4) {
    let uv = vec2(((idx << 1) & 2) as f32, (idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *coord = pos.extend(0.0).extend(1.0);
}
