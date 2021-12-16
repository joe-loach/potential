#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextSize {
    val: u32,
}

impl From<u32> for TextSize {
    #[inline]
    fn from(val: u32) -> Self {
        Self { val }
    }
}

impl From<TextSize> for u32 {
    #[inline]
    fn from(size: TextSize) -> Self {
        size.val
    }
}

impl TryFrom<usize> for TextSize {
    type Error = std::num::TryFromIntError;
    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(u32::try_from(value)?.into())
    }
}

impl From<TextSize> for usize {
    #[inline]
    fn from(size: TextSize) -> Self {
        size.val as usize
    }
}

use std::ops::*;

impl Add for TextSize {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            val: self.val + rhs.val,
        }
    }
}

impl AddAssign for TextSize {
    fn add_assign(&mut self, rhs: Self) {
        self.val += rhs.val
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextRange {
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    pub fn new(start: TextSize, end: TextSize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub fn at(start: TextSize, len: TextSize) -> Self {
        Self::new(start, start + len)
    }

    #[inline]
    pub const fn start(self) -> TextSize {
        self.start
    }

    #[inline]
    pub const fn end(self) -> TextSize {
        self.end
    }
}
