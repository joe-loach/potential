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
#[allow(unused_imports)]
use crate::spirv_std::num_traits::Float;

use spirv_std::glam::*;

use common::*;
use particle::*;

fn norm(a: f32, b: f32, t: f32) -> f32 {
    let t = t.min(b).max(a); // [a, b]
    (t - a) / (b - a) // [0, 1]
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + t * b
}

#[spirv(fragment)]
pub fn field(
    #[spirv(frag_coord)] pos: Vec4,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] constants: &ShaderConstants,
    #[spirv(uniform, descriptor_set = 0, binding = 1)] particles: &[Particle; 32],
    #[spirv(uniform, descriptor_set = 0, binding = 2)] colors: &[ColorVal; 2],
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
    let t = match constants.field {
        Field::Distance => particle::dist(pos, particles, len).unwrap_or(0.0),
        Field::Potential => particle::potential(pos, particles, len),
        Field::Force => particle::force(pos, particles, len),
    };
    let a = colors[0];
    let b = colors[1];
    let t = norm(a.val, b.val, t);
    let r = lerp(a.r, b.r, t);
    let g = lerp(a.g, b.g, t);
    let b = lerp(a.b, b.b, t);
    *output = vec4(r, g, b, 1.0);
}

#[spirv(vertex)]
pub fn vert(#[spirv(vertex_index)] idx: i32, #[spirv(position)] coord: &mut Vec4) {
    let uv = vec2(((idx << 1) & 2) as f32, (idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *coord = pos.extend(0.0).extend(1.0);
}
