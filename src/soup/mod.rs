//! The primordial soup: a 2D environment where amino acid chains are generated,
//! drift with temperature-dependent Brownian motion, mutate and recombine, and
//! where we watch for a target pattern ("METHINKS") to emerge.

mod census;
mod generation;
mod movement;
mod mutation;
mod recombination;
#[cfg(test)]
mod tests;

use crate::events::EventLog;
use crate::model::{Molecule, Position};
use rand::{SeedableRng, rngs::StdRng};

/// Population cap for chains (generation) and for the whole soup (recombination).
pub const MAX_AMINO_ACIDS: usize = 50;
/// The sequence we watch for.
pub const TARGET_PATTERN: &str = "METHINKS";

pub struct PrimordialSoup {
    pub molecules: Vec<Molecule>,
    pub temperature: f64,
    pub ph: f64,
    /// How every chain in the soup came to be. Chains added with `add_molecule`
    /// or by writing `molecules` directly may have no entry, so readers must
    /// treat a missing creation event as normal.
    pub events: EventLog,
    /// Steps simulated so far. Seed chains are logged at step 0; events during
    /// the n-th `simulate_step` are logged at step n.
    pub step: u64,
    width: f64,
    height: f64,
    total_reactions: u64,
    reactions_this_step: u64,
    rng: StdRng,
}

impl PrimordialSoup {
    pub fn new(width: f64, height: f64, temperature: f64, ph: f64) -> Self {
        Self::with_rng(width, height, temperature, ph, StdRng::from_os_rng())
    }

    pub fn with_seed(width: f64, height: f64, temperature: f64, ph: f64, seed: u64) -> Self {
        Self::with_rng(width, height, temperature, ph, StdRng::seed_from_u64(seed))
    }

    fn with_rng(width: f64, height: f64, temperature: f64, ph: f64, rng: StdRng) -> Self {
        let mut soup = Self {
            molecules: Vec::new(),
            temperature,
            ph,
            events: EventLog::new(),
            step: 0,
            width,
            height,
            total_reactions: 0,
            reactions_this_step: 0,
            rng,
        };
        for _ in 0..generation::INITIAL_AMINO_ACIDS {
            soup.spawn_chain();
        }
        soup
    }

    /// The molecule simulation from the launcher: 20 seed chains plus 50 water and
    /// 50 methane molecules at 300 K, pH 7.
    pub fn standard(width: f64, height: f64) -> Self {
        let mut soup = Self::new(width, height, 300.0, 7.0);
        soup.add_small_molecules();
        soup
    }

    /// Adds a molecule if its position is inside the soup.
    pub fn add_molecule(&mut self, molecule: Molecule) {
        if self.is_valid_position(&molecule.position) {
            self.molecules.push(molecule);
        }
    }

    fn is_valid_position(&self, p: &Position) -> bool {
        p.x >= 0.0 && p.x < self.width && p.y >= 0.0 && p.y < self.height
    }

    /// Advances one step: generation, mutation, recombination, movement. Returns
    /// the sequences that contain the target pattern after the step.
    pub fn simulate_step(&mut self) -> Vec<String> {
        self.step += 1;
        self.reactions_this_step = 0;
        self.generate_amino_acids();
        self.mutate();
        self.recombine();
        self.drift();
        self.target_matches()
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    /// Reactions are not modelled yet; always 0.
    pub fn total_reactions(&self) -> u64 {
        self.total_reactions
    }

    /// Reactions are not modelled yet; always 0.
    pub fn reactions_this_step(&self) -> u64 {
        self.reactions_this_step
    }
}
