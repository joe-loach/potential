mod particle;

pub mod graph;
pub mod scientific;

pub use particle::*;

use glam::Vec2;

pub fn map_pos(pos: Vec2, res: Vec2, x_axis: Vec2, y_axis: Vec2) -> Vec2 {
    // [0, a]
    // [0, 1]       (/a)
    // [0, c-b]     (*(c-b))
    // [b, c]       + b
    fn map(x: f32, a: f32, b: f32, c: f32) -> f32 {
        (x / a) * (c - b) + b
    }
    let (x, y) = (pos.x, pos.y);
    let x = map(x, res.x, x_axis.x, x_axis.y);
    let y = map(y, res.y, y_axis.y, y_axis.x);
    Vec2::new(x, y)
}