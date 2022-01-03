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

use particle::Particle;
use spirv_std::glam::*;

fn potential_sum(pos: Vec2, particles: &[Particle]) -> f32 {
    let mut idx = 0;
    let mut v = 0.0;
    while idx < particles.len() {
        let p = &particles[idx];
        match p.potential(pos) {
            Ok(x) => v += x,
            Err(x) => {
                return v + x;
            }
        }
        idx += 1;
    }
    v
}

#[spirv(fragment)]
pub fn frag(
    #[spirv(frag_coord)] coord: Vec4,
    output: &mut Vec4
) {
    let coord = coord.xy();
    // let v = potential_sum(coord, &particles);
    let particle = Particle::new(1.0, 1.0, vec2(0.0, 0.0));
    let v = match particle.potential(coord) {
        Ok(x) => x,
        Err(x) => x,
    };
    let v = v.abs().clamp(0.0, 1.0);
    *output = vec4(v, v, v, 1.0);
}

#[spirv(vertex)]
pub fn vert(#[spirv(vertex_index)] idx: i32, #[spirv(position)] coord: &mut Vec4) {
    let uv = vec2(((idx << 1) & 2) as f32, (idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;
    *coord = pos.extend(0.0).extend(1.0);
}