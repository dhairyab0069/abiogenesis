//! The geometry and timelines behind the cell drawings, with no egui in it.
//!
//! Design choices:
//! - **Everything is a function of numbers the simulation already has.** A cell's
//!   radius, its age in steps, where the lipids were when it formed, where a
//!   swallowed cell was: the drawing never keeps a story of its own, so a paused
//!   soup holds still and a replayed seed looks the same.
//! - **A membrane is made of lipid glyphs.** Two rings (leaflets) of lipids; in
//!   each, a round polar head and two tails. The outer leaflet has its heads out,
//!   the inner one has its heads in, and the tails of both meet in the middle.
//! - **Level of detail.** Only a cell that is big enough on screen gets glyphs, and
//!   the glyph count per leaflet and per frame is capped, so drawing stays cheap.
//!   Small cells get a smooth double line.
//! - **Formation is interpolation.** Each lipid glyph travels from where its real
//!   lipid was to its place on the ring, turning its head outward on the way; the
//!   arc grows as glyphs arrive, curls, closes into a vesicle, and only then does
//!   the inside fill. Division is two circles moving apart (the waist narrows by
//!   itself); engulfing is an arc reaching around the guest, closing, and the guest
//!   shrinking into the host's mitochondrion slot.

use std::f32::consts::{PI, TAU};
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pt {
    pub x: f32,
    pub y: f32,
}

impl Pt {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// The point `len` away in direction `angle`.
    pub fn polar(angle: f32, len: f32) -> Self {
        Self::new(angle.cos() * len, angle.sin() * len)
    }

    pub fn lerp(self, other: Pt, t: f32) -> Pt {
        self + (other - self) * t
    }

    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn dist(self, other: Pt) -> f32 {
        (self - other).len()
    }
}

impl Add for Pt {
    type Output = Pt;
    fn add(self, o: Pt) -> Pt {
        Pt::new(self.x + o.x, self.y + o.y)
    }
}

impl Sub for Pt {
    type Output = Pt;
    fn sub(self, o: Pt) -> Pt {
        Pt::new(self.x - o.x, self.y - o.y)
    }
}

impl Mul<f32> for Pt {
    type Output = Pt;
    fn mul(self, k: f32) -> Pt {
        Pt::new(self.x * k, self.y * k)
    }
}

/// Smooth 0 to 1 (clamped).
pub fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A fixed pseudo-random number in 0..1 from two integers.
pub fn hash01(seed: u64, k: u64) -> f32 {
    let mut z = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(k.wrapping_mul(0xBF58_476D_1CE4_E5B9)).wrapping_add(0x94D0_49BB_1331_11EB);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

/// The shortest signed turn from angle `a` to angle `b`.
fn turn(a: f32, b: f32) -> f32 {
    (b - a + PI).rem_euclid(TAU) - PI
}

// ---------------------------------------------------------------- the bilayer

/// The most lipids in one leaflet.
pub const MAX_PER_LEAFLET: usize = 48;
const MIN_PER_LEAFLET: usize = 10;
/// A cell this many pixels wide (radius) or more gets individual lipid glyphs.
pub const GLYPH_LOD_RADIUS: f32 = 22.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaflet {
    Outer,
    Inner,
}

/// One lipid: a round head and two tails, relative to the cell centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub leaflet: Leaflet,
    /// Direction from the cell centre to the glyph.
    pub angle: f32,
    pub head: Pt,
    pub head_radius: f32,
    /// Each tail from just inside the head to the middle of the membrane.
    pub tails: [(Pt, Pt); 2],
}

/// The sizes of a membrane around a cell of `radius`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bilayer {
    pub head_radius: f32,
    pub tail_len: f32,
    /// The radius of the plane where the two leaflets' tails meet.
    pub mid: f32,
    /// The radius where the cytoplasm begins.
    pub inner_edge: f32,
}

pub fn bilayer_spec(radius: f32) -> Bilayer {
    let head_radius = (radius * 0.055).max(1.4);
    let tail_len = head_radius * 1.6;
    let half = tail_len + 2.0 * head_radius;
    Bilayer { head_radius, tail_len, mid: radius - half, inner_edge: radius - 2.0 * half }
}

fn leaflet_count(ring_radius: f32, head_radius: f32) -> usize {
    let n = (TAU * ring_radius / (2.0 * head_radius * 1.15)).round().max(0.0) as usize;
    n.clamp(MIN_PER_LEAFLET, MAX_PER_LEAFLET)
}

/// How many glyphs the two leaflets have for a cell of `radius`.
pub fn glyph_total(radius: f32) -> usize {
    let s = bilayer_spec(radius);
    leaflet_count(s.mid + s.tail_len + s.head_radius, s.head_radius) + leaflet_count(s.mid - s.tail_len - s.head_radius, s.head_radius)
}

fn make_glyph(s: &Bilayer, leaflet: Leaflet, angle: f32) -> Glyph {
    let dir = Pt::polar(angle, 1.0);
    let tangent = Pt::new(-dir.y, dir.x);
    let (head_r, start_r, end_r) = match leaflet {
        Leaflet::Outer => (s.mid + s.tail_len + s.head_radius, s.mid + s.tail_len + s.head_radius * 0.2, s.mid + s.head_radius * 0.1),
        Leaflet::Inner => (s.mid - s.tail_len - s.head_radius, s.mid - s.tail_len - s.head_radius * 0.2, s.mid - s.head_radius * 0.1),
    };
    let side = s.head_radius * 0.55;
    let tail = |k: f32| (dir * start_r + tangent * (side * k), dir * end_r + tangent * (side * k));
    Glyph { leaflet, angle, head: dir * head_r, head_radius: s.head_radius, tails: [tail(-1.0), tail(1.0)] }
}

/// All the glyphs of a membrane: the outer leaflet first, then the inner one
/// (offset by half a step so the heads do not line up).
pub fn bilayer(radius: f32) -> Vec<Glyph> {
    let s = bilayer_spec(radius);
    let outer = leaflet_count(s.mid + s.tail_len + s.head_radius, s.head_radius);
    let inner = leaflet_count(s.mid - s.tail_len - s.head_radius, s.head_radius);
    let mut glyphs: Vec<Glyph> = (0..outer).map(|i| make_glyph(&s, Leaflet::Outer, i as f32 / outer as f32 * TAU)).collect();
    glyphs.extend((0..inner).map(|i| make_glyph(&s, Leaflet::Inner, (i as f32 + 0.5) / inner as f32 * TAU)));
    glyphs
}

/// A small shimmer for glyph `i` at `time` seconds: (outward, sideways), each in -1..1.
pub fn wobble(i: usize, time: f32) -> (f32, f32) {
    let phase = i as f32 * 1.713;
    ((time * 2.1 + phase).sin(), (time * 1.3 + phase * 0.7).cos())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    Smooth,
    Glyphs,
}

/// Glyphs when the cell is big enough on screen and the frame's glyph budget
/// has room for it, else a smooth double line.
pub fn detail(radius_px: f32, budget_left: usize) -> Detail {
    if radius_px >= GLYPH_LOD_RADIUS && glyph_total(radius_px) <= budget_left { Detail::Glyphs } else { Detail::Smooth }
}

/// Points around a circle, split into runs wherever `skip` says a point is hidden
/// (inside a neighbour's cytoplasm, for instance). With nothing hidden the single
/// run is closed (first point equals last).
pub fn membrane_runs(centre: Pt, radius: f32, samples: usize, skip: impl Fn(Pt) -> bool) -> Vec<Vec<Pt>> {
    let mut runs: Vec<Vec<Pt>> = Vec::new();
    let mut open = false;
    for k in 0..=samples {
        let p = centre + Pt::polar(k as f32 / samples as f32 * TAU, radius);
        if skip(p) {
            open = false;
        } else if open {
            runs.last_mut().expect("a run is open").push(p);
        } else {
            runs.push(vec![p]);
            open = true;
        }
    }
    runs
}

/// The two points where circle 1 meets circle 2, if they overlap and neither holds the other.
pub fn circle_intersection(c1: Pt, r1: f32, c2: Pt, r2: f32) -> Option<(Pt, Pt)> {
    let d = c1.dist(c2);
    if d < 1e-4 || d >= r1 + r2 || d <= (r1 - r2).abs() {
        return None;
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let along = (c2 - c1) * (1.0 / d);
    let mid = c1 + along * a;
    let side = Pt::new(-along.y, along.x) * h;
    Some((mid + side, mid - side))
}

// ------------------------------------------------------------------ formation

/// Steps a cell takes to form from a lipid crowd, as drawn.
pub const FORMATION_STEPS: f32 = 150.0;

/// How far through its formation a cell of this age is, or `None` once it is a finished cell.
pub fn formation_progress(age_steps: f32) -> Option<f32> {
    (0.0..FORMATION_STEPS).contains(&age_steps).then(|| age_steps / FORMATION_STEPS)
}

/// When glyph `i` of `n` reaches its place on the ring: the ring fills in order.
pub fn arrival(i: usize, n: usize) -> f32 {
    0.08 + 0.52 * (i as f32 + 1.0) / n.max(1) as f32
}

/// The angle of ring that is filled at progress `t` (a growing arc, then a full circle).
pub fn arc_span(t: f32) -> f32 {
    TAU * ((t - 0.08) / 0.52).clamp(0.0, 1.0)
}

/// How much the arc is flattened: 1.6 at first, 1 once the vesicle is closed.
pub fn curl(t: f32) -> f32 {
    1.0 + 0.6 * (1.0 - ease((t - 0.1) / 0.6))
}

/// How visible the inner leaflet is: it appears once the sheet has closed round.
pub fn inner_alpha(t: f32) -> f32 {
    ease((t - 0.5) / 0.2)
}

/// How filled the vesicle is (cytoplasm and trapped contents): only after it has sealed.
pub fn fill_alpha(t: f32) -> f32 {
    ease((t - 0.7) / 0.3)
}

/// Where an outer-leaflet glyph is at progress `t`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphPose {
    pub pos: Pt,
    pub angle: f32,
    pub alpha: f32,
}

/// The pose of outer glyph `i` of `n`: it starts at `start` (where its lipid was,
/// relative to the cell centre) facing a random way and travels to `target`,
/// turning its head outward, arriving at [`arrival`].
pub fn outer_pose(i: usize, n: usize, start: Pt, seed: u64, target: &Glyph, t: f32) -> GlyphPose {
    let p = (t / arrival(i, n)).clamp(0.0, 1.0);
    let ring = target.head * curl(t);
    let a0 = hash01(seed, i as u64) * TAU;
    GlyphPose {
        pos: start.lerp(ring, ease(p)),
        angle: a0 + turn(a0, target.angle) * ease(p),
        alpha: 1.0,
    }
}

// ------------------------------------------------------------------ division

/// Steps the pinch of a dividing cell takes, as drawn.
pub const PINCH_STEPS: f32 = 100.0;

pub fn pinch_progress(age_steps: f32) -> Option<f32> {
    (0.0..PINCH_STEPS).contains(&age_steps).then(|| age_steps / PINCH_STEPS)
}

/// Where the two halves are at pinch progress `t`: both start at the midpoint
/// (one elongating cell) and ease apart to their real places.
pub fn pinch_centres(mother: Pt, daughter: Pt, t: f32) -> (Pt, Pt) {
    let mid = mother.lerp(daughter, 0.5);
    let k = ease(t);
    (mid.lerp(mother, k), mid.lerp(daughter, k))
}

/// The width of the neck between two circles: the chord where they overlap (the
/// whole small circle while they are still mostly one cell), 0 once they part.
pub fn neck_width(c1: Pt, r1: f32, c2: Pt, r2: f32) -> f32 {
    let (big, small) = (r1.max(r2), r1.min(r2));
    let widest_at = (big * big - small * small).max(0.0).sqrt();
    if c1.dist(c2) <= widest_at {
        return 2.0 * small;
    }
    circle_intersection(c1, r1, c2, r2).map_or(0.0, |(a, b)| a.dist(b))
}

/// The length of the dividing cell along its axis, from end to end.
pub fn pinch_length(c1: Pt, r1: f32, c2: Pt, r2: f32) -> f32 {
    c1.dist(c2) + r1 + r2
}

// ------------------------------------------------------------------ engulfing

/// Steps an engulfing takes, as drawn.
pub const ENGULF_STEPS: f32 = 110.0;
/// The cup has closed round the guest at this progress.
const CUP_CLOSED_AT: f32 = 0.45;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cup {
    /// How much of a ring the host's arm has drawn round the guest, in radians.
    pub span: f32,
    pub closed: bool,
    /// The guest's centre relative to the host's, and its radius.
    pub guest: Pt,
    pub guest_radius: f32,
    /// 0 while the guest is still itself, 1 once it is a mitochondrion.
    pub shrunk: f32,
}

/// The engulfing at `age` steps: `guest` and `guest_radius` are where and how big
/// the swallowed cell was; `slot` and `mito_radius` are where its mitochondrion ends up.
pub fn cup(age_steps: f32, guest: Pt, guest_radius: f32, slot: Pt, mito_radius: f32) -> Option<Cup> {
    if !(0.0..ENGULF_STEPS).contains(&age_steps) {
        return None;
    }
    let t = age_steps / ENGULF_STEPS;
    let reach = ease(t / CUP_CLOSED_AT);
    let moved = ease((t - CUP_CLOSED_AT) / (1.0 - CUP_CLOSED_AT));
    // before it closes the guest is drawn a little toward the host, as if pulled
    let pulled = guest * (1.0 - 0.15 * reach);
    Some(Cup {
        span: PI * 0.8 + (TAU - PI * 0.8) * reach,
        closed: t >= CUP_CLOSED_AT,
        guest: pulled.lerp(slot, moved),
        guest_radius: guest_radius + (mito_radius - guest_radius) * moved,
        shrunk: moved,
    })
}

// ------------------------------------------------------------------- contents

/// The radius of a nucleus in a cell of `radius`.
pub fn nucleus_radius(radius: f32) -> f32 {
    radius * 0.38
}

/// Where mitochondrion `i` sits (relative to the centre) and its half-length: in the
/// band of cytoplasm between the nucleus and the membrane.
pub fn mito_slot(i: u32, radius: f32) -> (Pt, f32) {
    let inner = bilayer_spec(radius).inner_edge;
    let nucleus = nucleus_radius(radius);
    (Pt::polar(0.9 + i as f32 * 2.1, (inner + nucleus) / 2.0), (inner - nucleus).max(0.0) * 0.45)
}

#[cfg(test)]
mod tests;
