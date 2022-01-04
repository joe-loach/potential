#[repr(C)]
#[derive(Clone, Copy)]
pub struct Axis {
    pub min: f32,
    pub max: f32,
}

impl Axis {
    pub fn new(min: f32, max: f32) -> Axis {
        assert!(min < max);
        Self { min, max }
    }
}

use core::ops::*;

impl Mul<f32> for Axis {
    type Output = Axis;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            min: self.min * rhs,
            max: self.max * rhs,
        }
    }
}

impl MulAssign<f32> for Axis {
    fn mul_assign(&mut self, rhs: f32) {
        self.min *= rhs;
        self.max *= rhs;
    }
}

impl Sub<f32> for Axis {
    type Output = Axis;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl SubAssign<f32> for Axis {
    fn sub_assign(&mut self, rhs: f32) {
        self.min -= rhs;
        self.max -= rhs;
    }
}
