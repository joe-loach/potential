pub struct Axis {
    pub min: f32,
    pub max: f32,
}

impl Axis {
    pub fn new(min: f32, max: f32) -> Axis {
        assert!(min < max);
        Self { min, max }
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }
}
