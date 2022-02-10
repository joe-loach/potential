pub struct Sci(pub f32);

use core::fmt;

impl fmt::Debug for Sci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_infinite() {
            return write!(f, "INF");
        }
        if self.0.is_nan() {
            return write!(f, "NAN");
        }
        let x = format!("{:+.2e}", self.0);
        let (x, y) = x.split_once('e').unwrap();
        let (sign, y) = match y.strip_prefix('-') {
            Some(y) => ('-', y),
            None => ('+', y),
        };
        write!(f, "{x}e{sign}{y:0>2}")?;
        Ok(())
    }
}

impl fmt::Display for Sci {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}