extern crate ultraviolet as uv;

mod context;
pub mod event;
mod helper;
pub mod poml;
pub mod shapes;
mod store;

pub use context::*;
pub use event::EventHandler;
pub use shapes::Shape;
pub use store::*;

/// Coulomb constant
// const K: f32 = 8.987_552e9;
/// Gravitational constant
// const G: f32 = 6.674_302e-11;

pub struct Potential(pub uv::Vec2);
pub struct Force(pub uv::Vec2);
pub struct Distance(pub f32);

pub trait Field<T> {
    fn at(&self, pos: uv::Vec2) -> T;
}

pub struct Object {
    pub value: f32,
    pub pos: uv::Vec2,
    pub shape: Shape,
}

impl Object {
    pub fn new(value: f32, pos: uv::Vec2, shape: Shape) -> Self {
        Self { value, pos, shape }
    }
}

impl Field<Distance> for Object {
    fn at(&self, pos: uv::Vec2) -> Distance {
        let pos = pos - self.pos;
        let d = match self.shape {
            Shape::Circle { radius } => pos.mag() - radius,
        };
        Distance(d)
    }
}

impl Field<Potential> for &[Object] {
    fn at(&self, pos: uv::Vec2) -> Potential {
        let v = self
            .iter()
            .map(|o| {
                let vec = pos - o.pos;
                let r = vec.mag();
                if o.at(pos).0 >= 0.0 {
                    Ok(o.value * vec / (r * r))
                } else {
                    let r = match o.shape {
                        Shape::Circle { radius } => radius,
                    };
                    Err(o.value * vec / (r * r))
                }
            })
            .fold(Ok(uv::Vec2::zero()), |a, b| match (a, b) {
                (Ok(a), Ok(b)) => Ok(a + b),
                (Err(a), _) => Err(a),
                (_, Err(a)) => Err(a),
            });
        Potential(match v {
            Ok(v) => v,
            Err(v) => v,
        })
    }
}

impl Field<Force> for &[Object] {
    fn at(&self, pos: uv::Vec2) -> Force {
        Force(
            self.iter()
                .map(|o| {
                    let vec = pos - o.pos;
                    let r = vec.mag();
                    if o.at(pos).0 >= 0.0 {
                        Some(o.value * vec / (r * r * r))
                    } else {
                        None
                    }
                })
                .fold(Some(uv::Vec2::zero()), |a, b| match (a, b) {
                    (Some(a), Some(b)) => Some(a + b),
                    _ => None,
                })
                .unwrap_or_else(uv::Vec2::zero),
        )
    }
}
