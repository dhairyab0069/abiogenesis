//! The heritable body of one animal: a body plan and eleven organ levels.
//!
//! Every organ is one number from 0 (absent) to 1 (fully developed). That is a
//! deliberate simplification: real organs come from many gene changes working
//! together, not from one dial. What the dials do give us is a body that can
//! grow in small steps, be inherited, be mutated and be drawn.

use crate::soup::Role;
use rand::Rng;

/// How many organ dials there are.
pub const ORGANS: usize = 11;

/// An organ or body part with a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Organ {
    /// Mouth and gut: a blind sac at first, a through-gut (mouth to anus) in a bilateral body.
    Gut,
    /// A nerve net becoming a nerve cord with a brain-like swelling at the head.
    Nerve,
    /// A patch of light-sensitive cells growing into an eye.
    Eyes,
    /// Muscle blocks along the body.
    Muscle,
    /// Fins: flaps that push against water.
    Fins,
    /// A stiffening rod (notochord) that later hardens into bone.
    Skeleton,
    /// Gills: breathing water.
    Gills,
    /// Hinged jaws that bite.
    Jaws,
    /// Lung-like organ (also the ancestor of a swim bladder): breathing air.
    Lungs,
    /// Limb-like fins: bony, jointed, able to press on the ground.
    Limbs,
    /// A sealed, waterproof skin.
    Skin,
}

impl Organ {
    /// Every organ, with each placed after the organs it grows from (a child's organs mutate in this order).
    pub const ALL: [Organ; ORGANS] = [
        Organ::Gut,
        Organ::Nerve,
        Organ::Eyes,
        Organ::Muscle,
        Organ::Fins,
        Organ::Skeleton,
        Organ::Gills,
        Organ::Jaws,
        Organ::Lungs,
        Organ::Limbs,
        Organ::Skin,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Organ::Gut => "mouth and gut",
            Organ::Eyes => "eyes",
            Organ::Nerve => "nerve cord and brain",
            Organ::Muscle => "muscle",
            Organ::Fins => "fins",
            Organ::Skeleton => "backbone",
            Organ::Jaws => "jaws",
            Organ::Gills => "gills",
            Organ::Lungs => "lungs",
            Organ::Limbs => "limbs",
            Organ::Skin => "sealed skin",
        }
    }

    /// What the organ costs to keep, per unit of level, in the fitness rule.
    pub fn upkeep(self) -> f32 {
        match self {
            Organ::Gut => 0.05,
            Organ::Eyes => 0.06,
            Organ::Nerve => 0.08,
            Organ::Muscle => 0.08,
            Organ::Fins => 0.04,
            Organ::Skeleton => 0.08,
            Organ::Jaws => 0.05,
            Organ::Gills => 0.05,
            Organ::Lungs => 0.07,
            Organ::Limbs => 0.08,
            Organ::Skin => 0.05,
        }
    }
}

/// Cost of keeping a bilateral body (a head end and a tail end).
pub const BILATERAL_UPKEEP: f32 = 0.01;
/// A radial body can only grow a blind gut this far...
pub const RADIAL_GUT_LIMIT: f32 = 0.3;
/// ... and a nerve net this far.
pub const RADIAL_NERVE_LIMIT: f32 = 0.25;
/// Chance per offspring and organ that the level changes.
pub const MUTATION_RATE: f32 = 0.12;
/// Size of a typical level change.
pub const MUTATION_STEP: f32 = 0.07;
/// Chance per offspring that radial becomes bilateral (the reverse is ten times rarer).
pub const SYMMETRY_RATE: f32 = 0.01;

/// One animal's heritable body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Traits {
    /// Bilateral (a head end and a tail end) rather than radial (round, no front).
    pub bilateral: bool,
    pub levels: [f32; ORGANS],
}

impl Traits {
    /// A plain radial colony-like animal with no organs.
    pub fn plain() -> Self {
        Self {
            bilateral: false,
            levels: [0.0; ORGANS],
        }
    }

    /// The ancestor handed on from stage 5: a colony with the given numbers of germ, soma
    /// and feeder cells. Feeder cells give it a first hint of a gut, soma cells a first hint
    /// of a protective outer layer. Everything else starts from nothing.
    pub fn from_colony(roles: &[Role]) -> Self {
        let share = |r: Role| {
            if roles.is_empty() {
                0.0
            } else {
                roles.iter().filter(|x| **x == r).count() as f32 / roles.len() as f32
            }
        };
        let mut t = Self::plain();
        t.set(Organ::Gut, 0.05 + 0.15 * share(Role::Feeder));
        t.set(Organ::Skin, 0.05 + 0.10 * share(Role::Soma));
        t
    }

    pub fn get(&self, organ: Organ) -> f32 {
        self.levels[organ.index()]
    }

    pub fn set(&mut self, organ: Organ, level: f32) {
        self.levels[organ.index()] = level.clamp(0.0, 1.0);
    }

    /// How far an organ may grow given the rest of the body. Development builds on
    /// earlier parts: eyes need a nerve to talk to, jaws need a skeleton and gills to
    /// grow from, lungs bud off the gut, limbs grow out of bony fins. The ceiling only
    /// limits *gaining* a level: an organ that is already higher stays.
    pub fn ceiling(&self, organ: Organ) -> f32 {
        let g = |o| self.get(o);
        match organ {
            Organ::Gut => {
                if self.bilateral {
                    1.0
                } else {
                    RADIAL_GUT_LIMIT
                }
            }
            Organ::Nerve => {
                if self.bilateral {
                    1.0
                } else {
                    RADIAL_NERVE_LIMIT
                }
            }
            Organ::Eyes => (g(Organ::Nerve) + 0.1).min(1.0),
            Organ::Muscle => (g(Organ::Nerve) + 0.3).min(1.0),
            Organ::Fins => {
                if self.bilateral {
                    (g(Organ::Muscle) + 0.1).min(1.0)
                } else {
                    0.1
                }
            }
            Organ::Skeleton => {
                if self.bilateral && g(Organ::Nerve) >= 0.3 {
                    1.0
                } else {
                    0.15
                }
            }
            Organ::Jaws => g(Organ::Gills).min(g(Organ::Skeleton)),
            Organ::Gills => 1.0,
            Organ::Lungs => g(Organ::Gut),
            Organ::Limbs => g(Organ::Fins).min(g(Organ::Skeleton)),
            Organ::Skin => 1.0,
        }
    }

    /// A child's body: this body with a few random changes (see [`MUTATION_RATE`]).
    pub fn offspring<R: Rng>(&self, rng: &mut R) -> Self {
        let mut child = *self;
        let flip = if self.bilateral {
            SYMMETRY_RATE / 10.0
        } else {
            SYMMETRY_RATE
        };
        if rng.random::<f32>() < flip {
            child.bilateral = !child.bilateral;
        }
        for organ in Organ::ALL {
            if rng.random::<f32>() >= MUTATION_RATE {
                continue;
            }
            let step = noise(rng) * MUTATION_STEP;
            let old = child.get(organ);
            let new = old + step;
            // gaining is limited by the ceiling, losing never is
            let new = if new > old {
                new.min(old.max(child.ceiling(organ)))
            } else {
                new
            };
            child.set(organ, new);
        }
        child
    }

    /// How many organ levels and body-plan points the body has in total: a crude "how developed".
    pub fn development(&self) -> f32 {
        self.levels.iter().sum::<f32>() + if self.bilateral { 1.0 } else { 0.0 }
    }

    /// A compact text form, for the CSV export: `r` or `b`, then eleven levels as hundredths.
    pub fn encode(&self) -> String {
        let mut s = String::from(if self.bilateral { "b" } else { "r" });
        for l in self.levels {
            s.push_str(&format!("-{:02}", (l * 100.0).round() as u32));
        }
        s
    }

    /// The inverse of [`Traits::encode`] (levels come back rounded to hundredths).
    pub fn decode(text: &str) -> Option<Self> {
        let mut parts = text.split('-');
        let bilateral = match parts.next()? {
            "b" => true,
            "r" => false,
            _ => return None,
        };
        let mut levels = [0.0; ORGANS];
        for l in &mut levels {
            let hundredths: u32 = parts.next()?.parse().ok()?;
            if hundredths > 100 {
                return None;
            }
            *l = hundredths as f32 / 100.0;
        }
        parts.next().is_none().then_some(Self { bilateral, levels })
    }
}

/// Roughly bell-shaped noise in about -1 to 1 (the sum of four uniform draws).
fn noise<R: Rng>(rng: &mut R) -> f32 {
    (0..4).map(|_| rng.random::<f32>() - 0.5).sum::<f32>() * 0.9
}

/// The average of a group of bodies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mean {
    /// Share of the group that is bilateral, 0 to 1.
    pub bilateral: f32,
    pub levels: [f32; ORGANS],
}

impl Mean {
    pub const ZERO: Mean = Mean {
        bilateral: 0.0,
        levels: [0.0; ORGANS],
    };

    pub fn of<'a, I: IntoIterator<Item = &'a Traits>>(group: I) -> Self {
        let mut sum = Self::ZERO;
        let mut n = 0.0;
        for t in group {
            n += 1.0;
            sum.bilateral += if t.bilateral { 1.0 } else { 0.0 };
            for (s, l) in sum.levels.iter_mut().zip(t.levels) {
                *s += l;
            }
        }
        if n == 0.0 {
            return Self::ZERO;
        }
        sum.bilateral /= n;
        for s in &mut sum.levels {
            *s /= n;
        }
        sum
    }

    pub fn get(&self, organ: Organ) -> f32 {
        self.levels[organ.index()]
    }

    /// The body to draw for this average: bilateral if most of the group is.
    pub fn typical(&self) -> Traits {
        Traits {
            bilateral: self.bilateral >= 0.5,
            levels: self.levels,
        }
    }
}
