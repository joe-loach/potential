pub struct Figure {
    pub width: f32,
    pub height: f32,
    pub x_min: f32,
    pub x_max: f32,
    pub y_min: f32,
    pub y_max: f32,
}

impl Figure {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            x_min: -1.0,
            x_max: 1.0,
            y_min: -1.0,
            y_max: 1.0,
        }
    }
}
