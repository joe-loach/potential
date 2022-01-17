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
pub fn field(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] constants: &ShaderConstants,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] particles: &[Particle; 32],
    output: &mut Vec4,
) {
    if constants.len == 0 {
        *output = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    let pos = pos.xy();
    let res = vec2(constants.width as f32, constants.height as f32);
    let pos = map_pos(pos, res, constants.x_axis, constants.y_axis);
    let len = constants.len as usize;
    let x = match constants.field {
        Field::Distance => particle::dist(pos, particles, len).unwrap_or(0.0),
        Field::Potential => particle::potential(pos, particles, len),
        Field::Force => particle::force(pos, particles, len),
    };
    let x = x.abs().clamp(0.0, 1.0);

    *output = vec4(x, x, x, 1.0);
}

#[spirv(vertex)]
pub fn vert(#[spirv(vertex_index)] idx: i32, #[spirv(position)] coord: &mut Vec4) {
    let uv = vec2(((idx << 1) & 2) as f32, (idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *coord = pos.extend(0.0).extend(1.0);
}
