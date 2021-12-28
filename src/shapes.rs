#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Shape {
    Circle { radius: f32 },
}

pub fn circle(radius: f32) -> Shape {
    Shape::Circle { radius }
}
