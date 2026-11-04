//! A frames-per-second counter.

use std::time::Instant;

/// Frames per second, averaged over one-second windows.
pub(super) struct Fps {
    window_start: Instant,
    frames: u32,
    pub(super) value: f64,
}

impl Default for Fps {
    fn default() -> Self {
        Self { window_start: Instant::now(), frames: 0, value: 0.0 }
    }
}

impl Fps {
    pub(super) fn frame(&mut self) {
        self.frames += 1;
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            self.value = self.frames as f64 / elapsed;
            self.frames = 0;
            self.window_start = Instant::now();
        }
    }
}
