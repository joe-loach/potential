pub use ultraviolet as uv;

pub trait Sdf {
    fn dist(&self, p: uv::Vec2) -> f32;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    Circle,
    Rectangle,
}

pub struct Object<'s> {
    pos: uv::Vec2,
    shape: &'s dyn Sdf,
}

impl Sdf for Object<'_> {
    #[inline]
    fn dist(&self, p: uv::Vec2) -> f32 {
        self.shape.dist(p - self.pos)
    }
}

pub struct Circle {
    radius: f32,
}

impl Sdf for Circle {
    #[inline]
    fn dist(&self, p: uv::Vec2) -> f32 {
        p.mag() - self.radius
    }
}
