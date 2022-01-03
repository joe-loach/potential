extern crate ultraviolet as uv;

/// Coulomb constant
// const K: f32 = 8.987_552e9;
/// Gravitational constant
// const G: f32 = 6.674_302e-11;

pub struct Potential(uv::Vec2);
impl std::ops::Deref for Potential {
    type Target = uv::Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Force(uv::Vec2);
impl std::ops::Deref for Force {
    type Target = uv::Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Distance(f32);
impl std::ops::Deref for Distance {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Field<T> {
    fn at(&self, pos: uv::Vec2) -> T;
}

pub struct Particle {
    pub value: f32,
    pub radius: f32,
    pub pos: uv::Vec2,
}

impl Particle {
    pub fn new(value: f32, pos: uv::Vec2, radius: f32) -> Self {
        Self { value, pos, radius }
    }
}

impl Field<Distance> for Particle {
    fn at(&self, pos: uv::Vec2) -> Distance {
        let pos = pos - self.pos;
        Distance(pos.mag() - self.radius)
    }
}

impl Field<Potential> for &[Particle] {
    fn at(&self, pos: uv::Vec2) -> Potential {
        let v = self
            .iter()
            .map(|o| {
                let vec = pos - o.pos;
                let r = vec.mag();
                if o.at(pos).0 >= 0.0 {
                    Ok(o.value * vec / (r * r))
                } else {
                    let r = o.radius;
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

impl Field<Force> for &[Particle] {
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
