#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use glam::Vec2;
use spirv_std::glam;

// USEFUL CONSTANTS
pub const COULOMB: f32 = 8.987_552e9;
pub const GRAVITATIONAL: f32 = 6.674_302e-11;

pub struct Particle {
    pub value: f32,
    pub radius: f32,
    pub pos: Vec2,
}

impl Particle {
    #[inline]
    pub fn new(value: f32, radius: f32, pos: Vec2) -> Self {
        Self { value, radius, pos }
    }

    pub fn dist(&self, pos: Vec2) -> f32 {
        (pos - self.pos).length() - self.radius
    }

    pub fn potential(&self, pos: Vec2) -> Result<f32, f32> {
        let vec = pos - self.pos;
        if self.dist(pos) >= 0.0 {
            let r = vec.length();
            Ok(self.value / r)
        } else {
            let r = self.radius;
            Err(self.value / r)
        }
    }

    pub fn force(&self, pos: Vec2) -> Option<f32> {
        let vec = pos - self.pos;
        if self.dist(pos) >= 0.0 {
            let r = vec.length();
            Some(self.value / (r * r))
        } else {
            None
        }
    }
}
