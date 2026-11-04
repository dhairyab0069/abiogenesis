//! Movement: Brownian drift with temperature, the pool that shrinks as it dries,
//! and resizing the soup.

use super::PrimordialSoup;
use rand::Rng;

const MOVEMENT_SPEED: f64 = 2.0;
/// The share of the soup's height that stays wet even when the pool is as dry as it gets.
const MIN_POOL: f64 = 0.15;
/// How fast (per step) a molecule left above the water line sinks back into the pool.
const SETTLE_SPEED: f64 = 3.0;
/// Molecules stranded above the water line barely move.
const STRANDED_MOBILITY: f64 = 0.3;

impl PrimordialSoup {
    /// The height of the water surface, measured down from the top of the soup.
    /// 0 when the soup is fully wet; it rises toward the bottom as the pool dries.
    pub fn water_line(&self) -> f64 {
        self.height * (1.0 - (MIN_POOL + (1.0 - MIN_POOL) * self.wetness))
    }

    /// Moves every molecule by a random step scaled by temperature, staying in bounds.
    /// A drier pool is thicker (slower), and anything left above the water line
    /// sinks into the pool, so drying concentrates the chains. A molecule inside a
    /// protocell cannot leave it.
    pub(super) fn drift(&mut self) {
        let thermal = MOVEMENT_SPEED * (self.temperature / 300.0);
        let wet_scale = thermal * (0.6 + 0.4 * self.wetness);
        let water_line = self.water_line();
        let (w, h) = (self.width, self.height);
        let Self { molecules, protocells, rng, .. } = self;
        for m in molecules.iter_mut() {
            let stranded = m.position.y < water_line;
            let scale = if stranded { thermal * STRANDED_MOBILITY } else { wet_scale };
            let dx = (rng.random::<f64>() - 0.5) * scale;
            let dy = (rng.random::<f64>() - 0.5) * scale + if stranded { SETTLE_SPEED } else { 0.0 };
            let home = protocells.iter().find(|p| p.contains(m.position.x, m.position.y));
            m.position.x = (m.position.x + dx).clamp(0.0, w);
            m.position.y = (m.position.y + dy).clamp(0.0, h);
            if let Some(p) = home {
                (m.position.x, m.position.y) = p.keep_inside(m.position.x, m.position.y);
            }
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
        for c in &mut self.cells {
            c.x *= sx;
            c.y *= sy;
        }
        self.width = width;
        self.height = height;
    }
}
