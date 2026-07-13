use qrforge_application::ClockPort;
use std::time::{Duration, Instant};

/// Monotonic process-local clock.
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    /// Creates a clock with a fresh monotonic origin.
    #[must_use]
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl ClockPort for SystemClock {
    fn now(&self) -> Duration {
        self.origin.elapsed()
    }
}
