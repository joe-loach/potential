#![cfg_attr(target_arch = "spirv", no_std, feature(lang_items))]

use glam::Vec2;

// USEFUL CONSTANTS
pub const COULOMB: f32 = 8.987_552e9;
pub const GRAVITATIONAL: f32 = 6.674_302e-11;

pub trait Field<T> {
    fn at(&self, pos: Vec2) -> T;
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Distance(pub f32);

impl<'a> Field<Distance> for &'a [Particle] {
    fn at(&self, pos: Vec2) -> Distance {
        let mut d = Distance(f32::INFINITY);
        for p in *self {
            d = Distance(p.dist(pos).0.min(d.0));
        }
        d
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Potential(pub Vec2);

impl<'a> Field<Potential> for &'a [Particle] {
    fn at(&self, pos: Vec2) -> Potential {
        let mut v = Potential(Vec2::ZERO);
        for p in *self {
            match p.potential(pos) {
                Ok(x) => v.0 += x.0,
                Err(x) => return x,
            }
        }
        v
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Force(pub Vec2);

impl<'a> Field<Force> for &'a [Particle] {
    fn at(&self, pos: Vec2) -> Force {
        let mut e = Force(Vec2::ZERO);
        for p in *self {
            match p.force(pos) {
                Some(x) => e.0 += x.0,
                None => return Force(Vec2::ZERO),
            }
        }
        e
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Particle {
    pub value: f32,
    pub radius: f32,
    pub pos: Vec2,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            value: 1.0,
            radius: 1.0,
            pos: Default::default(),
        }
    }
}

impl Particle {
    #[inline]
    pub fn new(value: f32, radius: f32, pos: Vec2) -> Self {
        Self { value, radius, pos }
    }

    pub fn dist(&self, pos: Vec2) -> Distance {
        Distance((pos - self.pos).length() - self.radius)
    }

    pub fn potential(&self, pos: Vec2) -> Result<Potential, Potential> {
        let vec = pos - self.pos;
        if self.dist(pos).0 >= 0.0 {
            let r = vec.length();
            Ok(Potential(vec * self.value / (r * r)))
        } else {
            let r = self.radius;
            Err(Potential(vec * self.value / (r * r)))
        }
    }

    pub fn force(&self, pos: Vec2) -> Option<Force> {
        let vec = pos - self.pos;
        if self.dist(pos).0 >= 0.0 {
            let r = vec.length();
            Some(Force(vec * self.value / (r * r * r)))
        } else {
            None
        }
    }
}
