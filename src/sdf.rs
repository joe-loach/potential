pub trait Sdf {
    fn dist(&self, p: uv::Vec2) -> f32;
}

pub struct Circle {
    radius: f32,
}

impl Circle {
    #[inline]
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Sdf for Circle {
    #[inline]
    fn dist(&self, p: uv::Vec2) -> f32 {
        p.mag() - self.radius
    }
}
