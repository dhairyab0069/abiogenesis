//! A whole run of the lineage, computed up front so it can be played, scrubbed and compared.
//!
//! Design choices:
//! - **A run is a pure function of (environment, seed).** The same inputs give the same
//!   frames, so the scrubber can go backwards and a what-if can be compared with the
//!   baseline without re-simulating anything it has already seen.
//! - **Individuals, not averages.** Each animal has a body and a zone. Each generation
//!   every animal has offspring in proportion to how well its body suits its zone and how
//!   crowded the zone is; offspring mutate (`Traits::offspring`) and a few wander into the
//!   next zone, where they have to do well with the body they have.
//! - **Selection is natural, not forced.** Nothing steers the lineage toward land. If the
//!   shore makes land a bad place, the lineage stays in the sea.

use super::fitness::fitness;
use super::milestones::{Milestone, SETTLED, reached};
use super::traits::{Mean, Traits};
use super::zone::{Environment, ZONES, Zone};
use crate::soup::Role;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Animals the whole shore can hold at most.
pub const CAPACITY: usize = 240;
/// A zone always has room for at least this many.
pub const MIN_ROOM: usize = 8;
/// The seed the app starts with. Any seed tells the same kind of story; this one reaches land by the end.
pub const DEFAULT_SEED: u64 = 2;
/// Generations in a full run.
pub const GENERATIONS: u32 = 1600;
/// Animals a frame keeps for drawing.
pub const SAMPLES: usize = 14;
/// Offspring per animal at fitness 1 in an empty zone.
const BIRTH_RATE: f32 = 3.0;
/// A fitness this good is a typical animal; fitter animals have disproportionately more offspring.
const REFERENCE: f32 = 0.15;
const SELECTION: f32 = 2.0;
/// Chance an offspring settles in the next zone toward land or toward the sea.
const MIGRATION: f32 = 0.05;
const START_ANIMALS: usize = 60;

/// One animal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animal {
    pub traits: Traits,
    pub zone: Zone,
}

/// One animal kept for drawing.
pub type Sample = Animal;

/// What a generation looked like.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub generation: u32,
    /// Animals in each zone.
    pub counts: [u16; ZONES],
    /// The most landward zone with at least [`SETTLED`] animals.
    pub frontier_zone: Zone,
    /// The average body in that zone.
    pub frontier: Mean,
    /// The average body of everyone.
    pub overall: Mean,
    /// A spread of the living animals, for drawing.
    pub samples: Vec<Sample>,
}

impl Frame {
    pub fn total(&self) -> u32 {
        self.counts.iter().map(|c| *c as u32).sum()
    }
}

/// A finished run.
#[derive(Clone, Debug)]
pub struct Run {
    pub env: Environment,
    pub seed: u64,
    pub frames: Vec<Frame>,
    pub milestones: Vec<(Milestone, u32)>,
    /// The generation the whole lineage died out, if it did (a hypothetical world can be too hard).
    pub extinct_at: Option<u32>,
}

/// The numbers a what-if compares.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub env: Environment,
    pub generations: u32,
    pub counts: [u16; ZONES],
    pub frontier_zone: Zone,
    pub frontier: Mean,
    pub milestones: Vec<(Milestone, u32)>,
    pub extinct_at: Option<u32>,
}

impl Run {
    /// A run from the stage 5 ancestor (one germ, one soma and one feeder cell).
    pub fn new(env: Environment, seed: u64, generations: u32) -> Self {
        Self::from_ancestor(
            env,
            seed,
            generations,
            Traits::from_colony(&[Role::Germ, Role::Soma, Role::Feeder]),
        )
    }

    pub fn from_ancestor(env: Environment, seed: u64, generations: u32, ancestor: Traits) -> Self {
        let env = env.clamped();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut herd: Vec<Animal> = (0..START_ANIMALS)
            .map(|_| Animal {
                traits: ancestor,
                zone: Zone::OpenSea,
            })
            .collect();
        let mut extinct_at = None;
        let mut frames = Vec::with_capacity(generations as usize + 1);
        frames.push(frame_of(0, &herd));
        for generation in 1..=generations {
            herd = next_generation(&herd, &env, &mut rng);
            if herd.is_empty() && extinct_at.is_none() {
                extinct_at = Some(generation);
            }
            frames.push(frame_of(generation, &herd));
        }
        let milestones = reached(&frames);
        Self {
            env,
            seed,
            frames,
            milestones,
            extinct_at,
        }
    }

    pub fn last(&self) -> &Frame {
        self.frames
            .last()
            .expect("a run has at least its first frame")
    }

    /// The frame at `generation`, or the last one if the run is shorter.
    pub fn frame(&self, generation: u32) -> &Frame {
        self.frames
            .get(generation as usize)
            .unwrap_or_else(|| self.last())
    }

    /// Milestones reached up to and including `generation`.
    pub fn milestones_by(&self, generation: u32) -> Vec<(Milestone, u32)> {
        self.milestones
            .iter()
            .copied()
            .filter(|(_, g)| *g <= generation)
            .collect()
    }

    pub fn summary(&self) -> Summary {
        let last = self.last();
        Summary {
            env: self.env,
            generations: last.generation,
            counts: last.counts,
            frontier_zone: last.frontier_zone,
            frontier: last.frontier,
            milestones: self.milestones.clone(),
            extinct_at: self.extinct_at,
        }
    }

    /// The run as CSV text: one row per generation.
    pub fn to_csv(&self) -> String {
        let mut out =
            String::from("generation,open_sea,shallows,tidal_flats,land,frontier_zone,bilateral");
        for organ in super::traits::Organ::ALL {
            out.push(',');
            out.push_str(&organ.label().replace(' ', "_"));
        }
        out.push_str(",frontier_body\n");
        for f in &self.frames {
            out.push_str(&format!(
                "{},{},{},{},{},{}",
                f.generation,
                f.counts[0],
                f.counts[1],
                f.counts[2],
                f.counts[3],
                f.frontier_zone.label().replace(' ', "_")
            ));
            out.push_str(&format!(",{:.3}", f.frontier.bilateral));
            for l in f.frontier.levels {
                out.push_str(&format!(",{l:.3}"));
            }
            out.push_str(&format!(",{}\n", f.frontier.typical().encode()));
        }
        out
    }
}

/// How many animals a zone has room for.
pub fn room(env: &Environment, zone: Zone) -> usize {
    ((CAPACITY as f32 * env.shares()[zone.index()]).round() as usize).max(MIN_ROOM)
}

fn next_generation<R: Rng>(herd: &[Animal], env: &Environment, rng: &mut R) -> Vec<Animal> {
    let mut crowd = [0usize; ZONES];
    for a in herd {
        crowd[a.zone.index()] += 1;
    }
    let mut next: Vec<Animal> = Vec::with_capacity(herd.len() * 2);
    for a in herd {
        let z = a.zone;
        let load = crowd[z.index()] as f32 / room(env, z) as f32;
        let expected = BIRTH_RATE
            * (fitness(&a.traits, z, env).total() / REFERENCE).powf(SELECTION)
            / (1.0 + load);
        let mut n = expected.floor() as usize;
        if rng.random::<f32>() < expected.fract() {
            n += 1;
        }
        for _ in 0..n {
            let mut zone = z;
            if rng.random::<f32>() < MIGRATION
                && let Some(moved) = z.neighbour(rng.random_bool(0.5))
            {
                zone = moved;
            }
            next.push(Animal {
                traits: a.traits.offspring(rng),
                zone,
            });
        }
    }
    next.shuffle(rng);
    // trim each zone to its room (the shuffle makes the cull random)
    let mut kept = [0usize; ZONES];
    next.retain(|a| {
        let i = a.zone.index();
        kept[i] += 1;
        kept[i] <= room(env, a.zone)
    });
    next
}

fn frame_of(generation: u32, herd: &[Animal]) -> Frame {
    let mut counts = [0u16; ZONES];
    for a in herd {
        counts[a.zone.index()] += 1;
    }
    let frontier_zone = Zone::ALL
        .iter()
        .rev()
        .copied()
        .find(|z| counts[z.index()] >= SETTLED)
        .unwrap_or(Zone::OpenSea);
    let frontier = Mean::of(
        herd.iter()
            .filter(|a| a.zone == frontier_zone)
            .map(|a| &a.traits),
    );
    let overall = Mean::of(herd.iter().map(|a| &a.traits));
    // a stride over the herd sorted by zone keeps every zone in the picture
    let mut sorted: Vec<&Animal> = herd.iter().collect();
    sorted.sort_by(|a, b| {
        a.zone
            .index()
            .cmp(&b.zone.index())
            .then(a.traits.development().total_cmp(&b.traits.development()))
    });
    let samples = if sorted.len() <= SAMPLES {
        sorted.into_iter().copied().collect()
    } else {
        (0..SAMPLES)
            .map(|k| *sorted[k * sorted.len() / SAMPLES])
            .collect()
    };
    Frame {
        generation,
        counts,
        frontier_zone,
        frontier,
        overall,
        samples,
    }
}
