use instant::{Duration, Instant};

pub enum Timer {
    Started {
        start: Instant,
        last: Option<Instant>,
        curr: Option<Instant>,
    },
    Stopped,
}

impl Timer {
    pub fn new() -> Self {
        Timer::Stopped
    }

    pub fn start(&mut self) {
        *self = Timer::Started {
            start: Instant::now(),
            last: None,
            curr: None,
        };
    }

    pub fn tick(&mut self) -> Duration {
        match self {
            Timer::Started { start, last, curr } => {
                let now = Instant::now();
                let last = core::mem::replace(last, *curr);
                *curr = Some(now);
                let last = last.unwrap_or(*start);
                now.duration_since(last)
            }
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn delta(&self) -> Duration {
        match self {
            Timer::Started { start, last, curr } => {
                let last = last.unwrap_or(*start);
                let curr = curr.unwrap_or_else(Instant::now);
                curr.duration_since(last)
            }
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self {
            Timer::Started { start, .. } => start.elapsed(),
            Timer::Stopped => Duration::ZERO,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn timing() {
    let mut t = Timer::new();

    assert!(t.tick().is_zero());
    assert!(t.delta().is_zero());
    assert!(t.elapsed().is_zero());

    t.start();

    assert!(!t.elapsed().is_zero());
    let dt = t.tick();
    assert!(!dt.is_zero());
    assert_eq!(dt, t.delta());
}
