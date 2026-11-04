//! The shoreline scene as numbers: where the zones lie, how deep the ground is and where animals go.

use super::{Pt, hash01};
use crate::animals::{Environment, Organ, Traits, ZONES, Zone};

/// How an animal of some zone is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Posture {
    /// Swimming free in the water.
    Swim,
    /// On the bottom or the mud, propped on its fins or limbs, belly low.
    Prop,
    /// Standing on its limbs, belly lifted.
    Stand,
    /// Stranded: lying on its side, not coping.
    Stranded,
}

/// How an animal with `traits` is shown in `zone`.
pub fn posture(traits: &Traits, zone: Zone) -> Posture {
    let limbs = traits.get(Organ::Limbs);
    match zone {
        Zone::OpenSea => Posture::Swim,
        Zone::Shallows => {
            if limbs >= 0.35
                && traits.get(Organ::Skeleton) >= 0.4
                && limbs > traits.get(Organ::Fins) * 0.5
            {
                Posture::Prop
            } else {
                Posture::Swim
            }
        }
        Zone::TidalFlats => {
            if limbs >= 0.5 {
                Posture::Stand
            } else if limbs >= 0.25 {
                Posture::Prop
            } else {
                Posture::Stranded
            }
        }
        Zone::Land => {
            if limbs >= 0.5 {
                Posture::Stand
            } else if limbs >= 0.25 {
                Posture::Prop
            } else {
                Posture::Stranded
            }
        }
    }
}

/// Fractions of the scene's width and height: x from the open sea (0) to inland (1), y from the top (0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shore {
    /// Where each zone ends, as a fraction of the width.
    pub edges: [f32; ZONES],
}

/// Height (fraction of the scene) of the water surface at mean tide.
pub const WATER_LEVEL: f32 = 0.42;
/// How far the tide moves the surface, up and down.
pub const TIDE_RANGE: f32 = 0.035;
/// Height of the dry ground: above the highest tide.
pub const LAND_LEVEL: f32 = 0.36;
/// Height of the sea floor far out.
pub const DEEP_LEVEL: f32 = 0.93;

/// One animal's place in the scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// Where its centre is, as fractions of the scene.
    pub at: Pt,
    /// +1 facing inland (right), -1 facing the sea.
    pub facing: f32,
    /// A phase for its own motion (undulation, stepping).
    pub phase: f32,
}

impl Shore {
    pub fn new(env: &Environment) -> Self {
        let shares = env.shares();
        let mut edges = [0.0; ZONES];
        let mut acc = 0.0;
        for (e, s) in edges.iter_mut().zip(shares) {
            acc += s;
            *e = acc;
        }
        edges[ZONES - 1] = 1.0;
        Self { edges }
    }

    /// The first and last x of `zone`.
    pub fn span(&self, zone: Zone) -> (f32, f32) {
        let i = zone.index();
        (if i == 0 { 0.0 } else { self.edges[i - 1] }, self.edges[i])
    }

    /// The zone at x (fraction of the width).
    pub fn zone_at(&self, x: f32) -> Zone {
        Zone::ALL
            .into_iter()
            .find(|z| x < self.edges[z.index()])
            .unwrap_or(Zone::Land)
    }

    /// The height of the ground at x: deep far out, rising across the shallows and flats to the land.
    pub fn ground(&self, x: f32) -> f32 {
        let (_, sea_end) = self.span(Zone::OpenSea);
        let (_, flats_end) = self.span(Zone::TidalFlats);
        let (_, shallows_end) = self.span(Zone::Shallows);
        if x <= sea_end {
            // the floor rises slowly toward the shallows
            return DEEP_LEVEL - 0.04 * x / sea_end.max(1e-3);
        }
        if x >= flats_end {
            return LAND_LEVEL;
        }
        let start = DEEP_LEVEL - 0.04;
        let mid = WATER_LEVEL + 0.08;
        if x <= shallows_end {
            let t = (x - sea_end) / (shallows_end - sea_end).max(1e-3);
            start + (mid - start) * t
        } else {
            let t = (x - shallows_end) / (flats_end - shallows_end).max(1e-3);
            mid + (LAND_LEVEL - mid) * t
        }
    }

    /// The height of the water surface with the tide at `time` (seconds): the flats flood and drain.
    pub fn water_level(&self, time: f32) -> f32 {
        WATER_LEVEL + TIDE_RANGE * (time * 0.45).sin()
    }

    /// Is the ground at x under water at `time`?
    pub fn flooded(&self, x: f32, time: f32) -> bool {
        self.ground(x) > self.water_level(time)
    }

    /// Where the `k`-th of `n` animals in `zone` is at `time`. Each has its own spot (stable for a given
    /// `k`, `n`) and wanders around it.
    pub fn place(&self, zone: Zone, k: usize, n: usize, time: f32, posture: Posture) -> Placement {
        let (x0, x1) = self.span(zone);
        let width = (x1 - x0).max(0.01);
        let slot = (k as f32 + 0.5) / n.max(1) as f32;
        let jitter = (hash01(11, k as u64 * 7 + zone.index() as u64) - 0.5) * 0.4 / n.max(1) as f32;
        let home = x0 + width * (slot + jitter).clamp(0.08, 0.92);
        let phase = hash01(5, k as u64 + 31 * zone.index() as u64) * std::f32::consts::TAU;
        let wander = match posture {
            Posture::Swim => 0.3 * width.min(0.25),
            Posture::Prop | Posture::Stand => 0.05 * width.min(0.25),
            Posture::Stranded => 0.0,
        };
        let speed = 0.35 + 0.3 * hash01(9, k as u64);
        let x = (home + wander * (time * speed + phase).sin()).clamp(x0 + 0.01, x1 - 0.01);
        let facing = if posture == Posture::Stranded || (time * speed + phase).cos() >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let ground = self.ground(x);
        let surface = self.water_level(time);
        let y = match posture {
            Posture::Swim => {
                let top = surface + 0.05;
                let bottom = (ground - 0.07).max(top + 0.01);
                let lane = hash01(3, k as u64 + 17 * zone.index() as u64);
                top + (bottom - top) * (0.15 + 0.7 * lane) + 0.012 * (time * 1.3 + phase).sin()
            }
            Posture::Prop | Posture::Stranded => ground - 0.03,
            Posture::Stand => ground - 0.075,
        };
        Placement {
            at: Pt::new(x, y),
            facing,
            phase,
        }
    }
}
