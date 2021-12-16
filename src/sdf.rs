pub trait Sdf {
    fn dist(&self, p: uv::Vec2) -> f32;
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
