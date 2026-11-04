//! Milestones the lineage can reach, checked against the frontier of the population.
//!
//! Each one matches a chapter of the real-history story (see `story.rs`). They are
//! detected from the average body of the *frontier*: the most landward zone that
//! holds at least a handful of animals. A milestone is "reached" the first time its
//! condition holds, and stays reached.

use super::run::Frame;
use super::traits::Organ;
use super::zone::Zone;

/// Animals needed in a zone for it to count as lived in (and as the frontier).
pub const SETTLED: u16 = 4;
/// Animals needed out of the water for the "leaving the water" and "first land animals" milestones.
pub const FOOTHOLD: u16 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Milestone {
    /// A bilateral body with a through-gut.
    SymmetryAndGut,
    /// Eyes and a nerve cord with a brain-like swelling.
    EyesAndNerveCord,
    /// A stiffening rod: the backbone begins.
    Backbone,
    /// Hinged jaws.
    Jaws,
    /// Fins with a bony, limb-like core.
    LimbLikeFins,
    /// Lung-like organs.
    Lungs,
    /// A real population living on the tidal flats or beyond.
    LeavingWater,
    /// A real population living on land, with limbs and lungs.
    FirstLandAnimals,
}

impl Milestone {
    pub const ALL: [Milestone; 8] = [
        Milestone::SymmetryAndGut,
        Milestone::EyesAndNerveCord,
        Milestone::Backbone,
        Milestone::Jaws,
        Milestone::LimbLikeFins,
        Milestone::Lungs,
        Milestone::LeavingWater,
        Milestone::FirstLandAnimals,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Milestone::SymmetryAndGut => "symmetry and a gut",
            Milestone::EyesAndNerveCord => "eyes and a nerve cord",
            Milestone::Backbone => "a backbone",
            Milestone::Jaws => "jaws",
            Milestone::LimbLikeFins => "fins become limb-like",
            Milestone::Lungs => "lungs",
            Milestone::LeavingWater => "leaving the water",
            Milestone::FirstLandAnimals => "first land animals",
        }
    }

    /// Where this milestone comes in the real-history story (0 is first).
    pub fn story_rank(self) -> usize {
        // the order of `ALL` is the order of the story
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }

    /// Is the milestone's condition true in `frame`?
    pub fn holds(self, frame: &Frame) -> bool {
        let f = &frame.frontier;
        let count = |zones: &[Zone]| zones.iter().map(|z| frame.counts[z.index()]).sum::<u16>();
        match self {
            Milestone::SymmetryAndGut => f.bilateral >= 0.6 && f.get(Organ::Gut) >= 0.4,
            Milestone::EyesAndNerveCord => {
                f.get(Organ::Eyes) >= 0.35 && f.get(Organ::Nerve) >= 0.45
            }
            Milestone::Backbone => f.get(Organ::Skeleton) >= 0.35,
            Milestone::Jaws => f.get(Organ::Jaws) >= 0.35,
            Milestone::LimbLikeFins => f.get(Organ::Limbs) >= 0.3 && f.get(Organ::Fins) >= 0.3,
            Milestone::Lungs => f.get(Organ::Lungs) >= 0.35,
            Milestone::LeavingWater => {
                count(&[Zone::TidalFlats, Zone::Land]) >= FOOTHOLD && f.get(Organ::Limbs) >= 0.2
            }
            Milestone::FirstLandAnimals => {
                frame.counts[Zone::Land.index()] >= FOOTHOLD
                    && f.get(Organ::Limbs) >= 0.4
                    && f.get(Organ::Lungs) >= 0.35
            }
        }
    }
}

/// Milestones reached by the end of `frames`, as (milestone, generation of the first frame it held), in the order reached.
pub fn reached(frames: &[Frame]) -> Vec<(Milestone, u32)> {
    let mut found: Vec<(Milestone, u32)> = Vec::new();
    for frame in frames {
        for m in Milestone::ALL {
            if !found.iter().any(|(x, _)| *x == m) && m.holds(frame) {
                found.push((m, frame.generation));
            }
        }
    }
    found.sort_by_key(|(m, g)| (*g, m.story_rank()));
    found
}

/// How the simulation's order compares with the story's, for one milestone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// It came at the same place in the sequence as in the story.
    Same,
    /// It came earlier than the story has it: this many places earlier.
    Earlier(usize),
    /// It came later: this many places later.
    Later(usize),
}

/// Compares where `milestone` came among the milestones `reached` so far with its place in the story.
/// Only the milestones reached are ranked, so a missing one does not make the others look early.
pub fn order_of(milestone: Milestone, reached: &[(Milestone, u32)]) -> Option<Order> {
    let sim_rank = reached.iter().position(|(m, _)| *m == milestone)?;
    let story_rank = reached
        .iter()
        .filter(|(m, _)| m.story_rank() < milestone.story_rank())
        .count();
    Some(match sim_rank.cmp(&story_rank) {
        std::cmp::Ordering::Equal => Order::Same,
        std::cmp::Ordering::Less => Order::Earlier(story_rank - sim_rank),
        std::cmp::Ordering::Greater => Order::Later(sim_rank - story_rank),
    })
}
