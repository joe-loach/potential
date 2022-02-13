#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]

extern crate spirv_std;

#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

#[allow(unused_imports)]
use crate::spirv_std::num_traits::Float;

use spirv_std::glam::{vec2, vec4, Vec2, Vec4, Vec4Swizzles};

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
    let ShaderConstants {
        field,
        len,
        width,
        height,
        x_axis,
        y_axis,
    } = *constants;
    if len == 0 {
        *output = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    let pos = pos.xy();
    let res = vec2(width as f32, height as f32);
    let pos = map_pos(pos, res, x_axis, y_axis);
    let len = len as usize;
    let t = match field {
        Field::Distance => particle::dist(pos, particles, len).unwrap(),
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
