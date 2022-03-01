use glam::{Vec2, Vec3};

#[derive(Clone, Copy)]
pub struct Linspace {
    from: f32,
    inc: f32,
    points: u32,
    i: u32,
}

impl Linspace {
    pub(crate) fn new(from: f32, to: f32, points: u32) -> Self {
        Self {
            from,
            inc: (to - from) / (points - 1) as f32,
            points,
            i: 0,
        }
    }
}

impl Iterator for Linspace {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            from,
            inc,
            points,
            i,
        } = self;
        let x = if i == points {
            None
        } else {
            Some(*from + *inc * (*i as f32))
        };
        *i += 1;
        x
    }
}

pub fn linspace(from: f32, to: f32, points: u32) -> Linspace {
    assert!(from < to);
    Linspace::new(from, to, points)
}

pub fn grid(xs: Linspace, ys: Linspace) -> impl Iterator<Item = Vec2> {
    xs.flat_map(move |x| std::iter::repeat(x).zip(ys).map(|(y, x)| Vec2::new(x, y)))
}

pub fn contour(_g: impl Iterator<Item = Vec3>) {
    // https://dmahr1.github.io/618-final/report.html
    todo!()
}

#[test]
fn api() {
    let x = linspace(-1.0, 1.0, 4);
    let y = linspace(-1.0, 1.0, 4);
    let g = grid(x, y);
    let p = crate::particle::Particle::new(0.1, 0.1, Vec2::ZERO);
    let v = g.map(|pos| {
        pos.extend(
            match p.potential(pos) {
                Ok(v) => v,
                Err(v) => v,
            }
            .0
            .length(),
        )
    });
    contour(v);
}
