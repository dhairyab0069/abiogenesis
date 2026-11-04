//! Movement: Brownian drift with temperature, and resizing the soup.

use super::PrimordialSoup;
use rand::Rng;

const MOVEMENT_SPEED: f64 = 2.0;

impl PrimordialSoup {
    /// Moves every molecule by a random step scaled by temperature, staying in bounds.
    pub(super) fn drift(&mut self) {
        let scale = MOVEMENT_SPEED * (self.temperature / 300.0);
        let (w, h) = (self.width, self.height);
        for m in &mut self.molecules {
            let dx = (self.rng.random::<f64>() - 0.5) * scale;
            let dy = (self.rng.random::<f64>() - 0.5) * scale;
            m.position.x = (m.position.x + dx).clamp(0.0, w);
            m.position.y = (m.position.y + dy).clamp(0.0, h);
        }
    }

    /// Resizes the soup, rescaling every molecule's position to match.
    pub fn resize(&mut self, width: f64, height: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let (sx, sy) = (width / self.width, height / self.height);
        for m in &mut self.molecules {
            m.position.x *= sx;
            m.position.y *= sy;
        }
        self.width = width;
        self.height = height;
    }
}
