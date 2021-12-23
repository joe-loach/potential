extern crate ultraviolet as uv;

mod context;
pub mod event;
mod helper;
pub mod poml;
pub mod sdf;
mod store;

use std::rc::Rc;

pub use context::*;
pub use event::EventHandler;
pub use sdf::Sdf;
pub use store::*;

/// Coulomb constant
// const K: f32 = 8.987_552e9;
/// Gravitational constant
// const G: f32 = 6.674_302e-11;

pub struct Potential(pub f32);
pub struct Force(pub f32);

pub trait Field<T> {
    fn at(&self, pos: uv::Vec2) -> T;
}

pub struct Object {
    value: f32,
    pos: uv::Vec2,
    shape: Index<Box<dyn Sdf>>,
    store: Rc<Store<Box<dyn Sdf>>>,
}

impl Object {
    pub fn new(
        value: f32,
        pos: uv::Vec2,
        shape: Index<Box<dyn Sdf>>,
        store: Rc<Store<Box<dyn Sdf>>>,
    ) -> Self {
        Self {
            value,
            pos,
            shape,
            store,
        }
    }
}

impl Field<Potential> for &[Object] {
    fn at(&self, pos: uv::Vec2) -> Potential {
        Potential(
            self.iter()
                .map(|o| {
                    let r = o.dist(pos);
                    if r >= 0.0 {
                        Some(o.value / r)
                    } else {
                        None
                    }
                })
                .fold(Some(0.0), |a, b| {
                    if let (Some(a), Some(b)) = (a, b) {
                        Some(a + b)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0),
        )
    }
}

impl Field<Force> for &[Object] {
    fn at(&self, pos: uv::Vec2) -> Force {
        Force(
            self.iter()
                .map(|o| {
                    let r = o.dist(pos);
                    if r >= 0.0 {
                        Some(o.value / (r * r))
                    } else {
                        None
                    }
                })
                .fold(Some(0.0), |a, b| {
                    if let (Some(a), Some(b)) = (a, b) {
                        Some(a + b)
                    } else {
                        None
                    }
                })
                .unwrap_or(0.0),
        )
    }
}

impl Sdf for Object {
    #[inline]
    fn dist(&self, p: uv::Vec2) -> f32 {
        //todo!(); //self.shape.dist(p - self.pos)
        let shape = self.store.get(&self.shape);
        shape.dist(p - self.pos)
    }
}
