//! The geometry behind the animal drawings, with no egui in it.
//!
//! Design choices:
//! - **The picture is a function of the body.** Which organs are drawn, how big they
//!   are and where they sit all come from a [`Traits`] (and a few numbers for motion), so a
//!   paused run holds still, the scrubber can go backwards, and the same lineage always
//!   looks the same.
//! - **Organs fade in, then grow.** Below a small level an organ is not drawn at all; its
//!   opacity ramps up to full over the next few hundredths and its size follows its level.
//!   That is what makes organs *appear* as the lineage evolves.
//! - **One body in local units.** A bilateral body is 1 long, with the head (and mouth) at
//!   x = +0.5, the tail at -0.5, the back toward -y and the belly toward +y. A radial body is
//!   round, 0.6 across, and has no front. The drawing scales and places it.
//! - **A fin and a limb are one thing at two ends.** [`limb_shape`] turns the fin and limb
//!   levels into a flap of webbing that stiffens, grows a joint or two and ends in toes,
//!   and [`limb_chain`] poses it, swimming or walking.
//! - **Scene layout is separate** ([`Shore`]): where the zones sit along the shoreline,
//!   how deep the water is, the tide, and where an animal of a given zone goes.

mod scene;

pub use scene::{
    DEEP_LEVEL, LAND_LEVEL, Placement, Posture, Shore, TIDE_RANGE, WATER_LEVEL, posture,
};

pub use crate::cellviz::{Pt, ease, hash01};

use crate::animals::{Organ, Traits};
use std::f32::consts::PI;

/// An organ is not drawn below this level.
pub const APPEAR_AT: f32 = 0.04;
/// And is fully opaque from this level.
pub const OPAQUE_AT: f32 = 0.2;
/// The backbone is drawn as a notochord (a flexible rod) from this level...
pub const NOTOCHORD_AT: f32 = 0.25;
/// ... and as bone (vertebrae) from this level.
pub const BONE_AT: f32 = 0.6;
/// Jaws are drawn as a closed mouth below this level.
pub const JAWS_OPEN_AT: f32 = 0.2;
/// An eye is a light patch below this level and an eye with a pupil above it.
pub const EYE_AT: f32 = 0.3;
/// A nerve cord has a brain-like swelling from this level.
pub const BRAIN_AT: f32 = 0.35;
/// The radial body's arms.
pub const ARMS: usize = 6;

/// How visible an organ at `level` is, 0 to 1: nothing, then a fade-in, then full.
pub fn visibility(level: f32) -> f32 {
    if level < APPEAR_AT {
        0.0
    } else {
        ease((level - APPEAR_AT) / (OPAQUE_AT - APPEAR_AT))
    }
}

/// How much of its full size an organ at `level` is drawn at (grows with its level, never zero once visible).
pub fn size(level: f32) -> f32 {
    if level < APPEAR_AT {
        0.0
    } else {
        0.35 + 0.65 * level.clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backbone {
    None,
    Notochord,
    Bone,
}

pub fn backbone(level: f32) -> Backbone {
    if level >= BONE_AT {
        Backbone::Bone
    } else if level >= NOTOCHORD_AT {
        Backbone::Notochord
    } else {
        Backbone::None
    }
}

/// The half-height of a bilateral body at `s` (0 at the tail, 1 at the snout), in body lengths.
pub fn half_height(t: &Traits, s: f32) -> f32 {
    let s = s.clamp(0.0, 1.0);
    let body = 0.115 + 0.045 * t.get(Organ::Muscle);
    // fullest about 40% of the way from the tail; pointed at the snout, a slim tail stalk
    let shape = s.powf(0.55) * (1.0 - s).powf(0.75) / 0.41;
    (body * shape).max(0.012 * s.min(0.2) * 5.0)
}

/// Sideways swing of the body at `s` for an undulation `phase`: the tail swings most.
pub fn undulation(t: &Traits, s: f32, phase: f32) -> f32 {
    let amp = 0.012 + 0.05 * t.get(Organ::Muscle);
    amp * (1.0 - s) * (phase + (1.0 - s) * 3.2).sin()
}

/// A point on the bilateral body: `s` from tail to snout, `v` from -1 (back) to 1 (belly).
pub fn body_point(t: &Traits, s: f32, v: f32, phase: f32) -> Pt {
    Pt::new(s - 0.5, half_height(t, s) * v + undulation(t, s, phase))
}

/// The closed outline of a bilateral body, going along the back from tail to snout and back along the belly.
pub fn outline(t: &Traits, phase: f32, segments: usize) -> Vec<Pt> {
    let n = segments.max(4);
    let mut pts: Vec<Pt> = (0..=n)
        .map(|k| body_point(t, k as f32 / n as f32, -1.0, phase))
        .collect();
    pts.extend(
        (0..=n)
            .rev()
            .map(|k| body_point(t, k as f32 / n as f32, 1.0, phase)),
    );
    pts
}

/// The radius of a radial body for a pulse `phase`: round with a gentle pulse.
pub fn radial_radius(_t: &Traits, phase: f32) -> f32 {
    0.27 * (1.0 + 0.05 * phase.sin())
}

/// An arm of a radial body: points from the rim outward, waving with `phase`.
pub fn arm(t: &Traits, k: usize, phase: f32, points: usize) -> Vec<Pt> {
    let angle = k as f32 / ARMS as f32 * 2.0 * PI;
    let r0 = radial_radius(t, phase);
    let len = 0.17 + 0.03 * t.get(Organ::Muscle);
    (0..=points.max(2))
        .map(|i| {
            let f = i as f32 / points.max(2) as f32;
            let sway = 0.35 * f * (phase * 1.3 + k as f32 + f * 4.0).sin();
            Pt::polar(angle + sway * 0.4, r0 + len * f)
        })
        .collect()
}

/// The gut as a tube along the body from the mouth to the vent: a line for a through-gut,
/// a short pouch for a blind one. Points go from the mouth backward.
pub fn gut_line(t: &Traits, phase: f32) -> Vec<Pt> {
    let level = t.get(Organ::Gut);
    if !t.bilateral {
        return Vec::new();
    }
    // a through-gut reaches the vent at the tail end as the level grows
    let reach = 0.35 + 0.55 * level;
    (0..=8)
        .map(|k| {
            let s = 0.95 - reach * k as f32 / 8.0;
            let wiggle = 0.012 * (k as f32 * 1.7).sin();
            let mut p = body_point(t, s, 0.25, phase);
            p.y += wiggle;
            p
        })
        .collect()
}

/// The nerve cord along the back, from the head to the tail. The brain swelling sits at its head end.
pub fn nerve_line(t: &Traits, phase: f32) -> Vec<Pt> {
    let reach = 0.25 + 0.65 * t.get(Organ::Nerve);
    (0..=8)
        .map(|k| body_point(t, 0.9 - reach * k as f32 / 8.0, -0.55, phase))
        .collect()
}

/// Where the brain-like swelling is, and its radius.
pub fn brain(t: &Traits, phase: f32) -> (Pt, f32) {
    (
        body_point(t, 0.86, -0.35, phase),
        0.018 + 0.04 * t.get(Organ::Nerve),
    )
}

/// The eyes: position and radius of the near eye (the far one is drawn on the same spot, a little fainter).
pub fn eye(t: &Traits, phase: f32) -> (Pt, f32) {
    (
        body_point(t, 0.84, -0.3, phase),
        0.014 + 0.035 * t.get(Organ::Eyes),
    )
}

/// The backbone: from head to tail, with the vertebrae's positions (empty before bone).
pub fn backbone_line(t: &Traits, phase: f32) -> (Vec<Pt>, Vec<Pt>) {
    let line: Vec<Pt> = (0..=10)
        .map(|k| body_point(t, 0.88 - 0.8 * k as f32 / 10.0, -0.28, phase))
        .collect();
    let ticks = if backbone(t.get(Organ::Skeleton)) == Backbone::Bone {
        line.iter().skip(1).copied().collect()
    } else {
        Vec::new()
    };
    (line, ticks)
}

/// Chevron-shaped muscle blocks along the body: how many are drawn grows with the muscle level.
pub fn muscle_blocks(t: &Traits, phase: f32) -> Vec<[Pt; 3]> {
    let level = t.get(Organ::Muscle);
    if level < APPEAR_AT {
        return Vec::new();
    }
    let n = (3.0 + 6.0 * level).round() as usize;
    (0..n)
        .map(|k| {
            let s = 0.2 + 0.55 * k as f32 / n as f32;
            [
                body_point(t, s + 0.03, 0.0, phase),
                body_point(t, s, -0.6, phase),
                body_point(t, s, 0.6, phase),
            ]
        })
        .collect()
}

/// Gill slits behind the head: a few short strokes from the back toward the belly.
pub fn gill_slits(t: &Traits, phase: f32) -> Vec<[Pt; 2]> {
    let n = 2 + (3.0 * t.get(Organ::Gills)).round() as usize;
    (0..n)
        .map(|k| {
            let s = 0.72 - 0.035 * k as f32;
            [
                body_point(t, s, -0.2, phase),
                body_point(t, s - 0.01, 0.35, phase),
            ]
        })
        .collect()
}

/// The lung (or swim bladder): centre and radii, tucked above the gut.
pub fn lung(t: &Traits, phase: f32) -> (Pt, Pt) {
    let l = t.get(Organ::Lungs);
    (
        body_point(t, 0.55, -0.15, phase),
        Pt::new(0.05 + 0.07 * l, 0.02 + 0.045 * l),
    )
}

/// The jaws: the upper and lower jaw as triangles from the hinge, opened by `bite` (0 closed, 1 wide).
/// Without jaws (level below [`JAWS_OPEN_AT`]) it is a plain mouth: a single short stroke and no teeth.
pub fn jaws(t: &Traits, bite: f32, phase: f32) -> JawShape {
    let l = t.get(Organ::Jaws);
    let hinge = body_point(t, 0.82, 0.12, phase);
    let mouth = body_point(t, 1.0, 0.1, phase);
    if l < JAWS_OPEN_AT {
        return JawShape {
            upper: [hinge, mouth, mouth],
            lower: [hinge, mouth, mouth],
            teeth: false,
        };
    }
    let open = bite.clamp(0.0, 1.0) * (0.04 + 0.08 * l);
    let reach = 0.04 + 0.05 * l;
    let tip = Pt::new(mouth.x + reach, mouth.y);
    JawShape {
        upper: [
            hinge,
            Pt::new(mouth.x, mouth.y - 0.01),
            Pt::new(tip.x, tip.y - open * 0.6),
        ],
        lower: [
            hinge,
            Pt::new(mouth.x, mouth.y + 0.01),
            Pt::new(tip.x, tip.y + open),
        ],
        teeth: l >= 0.45,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JawShape {
    pub upper: [Pt; 3],
    pub lower: [Pt; 3],
    pub teeth: bool,
}

/// The tail fin: two lobes from the tail end, bigger with the fin level.
pub fn tail_fin(t: &Traits, phase: f32) -> [Pt; 4] {
    let f = size(t.get(Organ::Fins));
    let base = body_point(t, 0.02, 0.0, phase);
    let swing = undulation(t, 0.0, phase);
    [
        base,
        Pt::new(base.x - 0.12 * f, base.y - 0.1 * f + swing),
        Pt::new(base.x - 0.07 * f, base.y + swing),
        Pt::new(base.x - 0.12 * f, base.y + 0.1 * f + swing),
    ]
}

/// The back fin: a ridge along the back.
pub fn dorsal_fin(t: &Traits, phase: f32) -> [Pt; 3] {
    let f = size(t.get(Organ::Fins));
    [
        body_point(t, 0.62, -1.0, phase),
        body_point(t, 0.42, -1.0 - 0.9 * f, phase),
        body_point(t, 0.3, -1.0, phase),
    ]
}

/// How a paired fin or limb looks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LimbShape {
    /// Length in body lengths.
    pub length: f32,
    /// 0 a fin (a flap), 1 a limb.
    pub blend: f32,
    /// How much webbing joins the structure to its fringe: 1 a flap of fin, 0 none.
    pub web: f32,
    /// Bends along the limb (0 a stiff flap, 1 an elbow, 2 an elbow and a wrist).
    pub joints: usize,
    /// The limb ends in toes.
    pub toes: bool,
}

/// What the paired appendages look like for fin and limb levels. A limb is a fin that has
/// stiffened, grown joints and lost its web.
pub fn limb_shape(fins: f32, limbs: f32) -> Option<LimbShape> {
    if fins < APPEAR_AT && limbs < APPEAR_AT {
        return None;
    }
    let blend = ease((limbs - 0.08) / 0.62);
    let strength = fins.max(limbs).clamp(0.0, 1.0);
    Some(LimbShape {
        length: (0.1 + 0.17 * strength) * (1.0 + 0.55 * blend),
        blend,
        web: (1.0 - blend).clamp(0.0, 1.0),
        joints: if blend < 0.25 {
            0
        } else if blend < 0.6 {
            1
        } else {
            2
        },
        toes: blend > 0.85,
    })
}

/// A limb (or fin) as a chain of points from where it attaches: the shoulder, the elbow (a
/// repeat of the shoulder for a stiff flap) and the tip. `walking` points it down and forward to the
/// ground; otherwise a fin sweeps back, swinging with `phase`.
pub fn limb_chain(shape: &LimbShape, attach: Pt, phase: f32, walking: bool) -> [Pt; 3] {
    let swing = phase.sin();
    if walking {
        let upper = shape.length * 0.5;
        let lower = shape.length * 0.5 + 0.01;
        let elbow = Pt::new(
            attach.x - 0.15 * upper + swing * 0.5 * upper,
            attach.y + 0.85 * upper,
        );
        let tip = Pt::new(
            elbow.x + swing * 0.6 * lower + 0.2 * lower,
            elbow.y + lower * 0.95,
        );
        let elbow = if shape.joints == 0 {
            attach.lerp(tip, 0.5)
        } else {
            elbow
        };
        [attach, elbow, tip]
    } else {
        let tip = Pt::new(
            attach.x - shape.length * (0.8 + 0.15 * swing),
            attach.y + shape.length * (0.55 - 0.2 * swing),
        );
        let bend = if shape.joints == 0 { 0.0 } else { 0.25 };
        let elbow = Pt::new(
            attach.x + (tip.x - attach.x) * 0.5,
            attach.y + (tip.y - attach.y) * 0.5 + bend * shape.length,
        );
        [attach, elbow, tip]
    }
}

/// The lowest point of an animal's body above the ground when standing on `limbs` of `shape`, in body lengths:
/// the height the whole body is held at (0 when it lies on its belly).
pub fn stand_height(shape: Option<&LimbShape>, limbs: f32, skeleton: f32) -> f32 {
    match shape {
        Some(s) if limbs >= 0.3 => {
            s.length * (0.55 + 0.3 * skeleton.clamp(0.0, 1.0)) * ease((limbs - 0.25) / 0.5)
        }
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests;
