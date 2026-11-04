//! The primordial soup: a 2D environment where amino acid chains are generated,
//! drift with temperature-dependent Brownian motion, mutate and recombine, and
//! where we watch for a target pattern ("METHINKS") to emerge.

mod census;
mod generation;
mod movement;
mod mutation;
mod protocell;
mod recombination;
#[cfg(test)]
mod tests;

use crate::environment::Environment;
use crate::events::EventLog;
use crate::model::{Molecule, Position};
use rand::{SeedableRng, rngs::StdRng};

pub use protocell::Protocell;

/// Population cap for chains (generation) and for the whole soup (recombination).
pub const MAX_AMINO_ACIDS: usize = 50;
/// The sequence we watch for.
pub const TARGET_PATTERN: &str = "METHINKS";

pub struct PrimordialSoup {
    pub molecules: Vec<Molecule>,
    /// The current conditions. The environment sets all three at the start of every
    /// step, so change them with `set_environment`, not by writing the fields.
    pub temperature: f64,
    pub ph: f64,
    /// 0.0 = the pool is dry, 1.0 = wet everywhere.
    pub wetness: f64,
    /// What the soup feels over time: constant, or cycling.
    pub environment: Environment,
    /// Droplets that formed where chains crowded as the pool dried.
    pub protocells: Vec<Protocell>,
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
    protocells_formed: u64,
    protocells_burst: u64,
    next_protocell_id: u64,
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
            wetness: 1.0,
            environment: Environment::constant(temperature, ph),
            protocells: Vec::new(),
            events: EventLog::new(),
            step: 0,
            width,
            height,
            total_reactions: 0,
            reactions_this_step: 0,
            protocells_formed: 0,
            protocells_burst: 0,
            next_protocell_id: 1,
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

    /// Replaces the environment and applies it right away.
    pub fn set_environment(&mut self, environment: Environment) {
        self.environment = environment;
        self.apply_environment();
    }

    fn apply_environment(&mut self) {
        let conditions = self.environment.at(self.step);
        self.temperature = conditions.temperature;
        self.ph = conditions.ph;
        self.wetness = conditions.wetness;
    }

    /// Advances one step: the environment, generation, mutation, recombination,
    /// movement, then protocells. Returns the sequences that contain the target
    /// pattern after the step.
    pub fn simulate_step(&mut self) -> Vec<String> {
        self.step += 1;
        self.reactions_this_step = 0;
        self.apply_environment();
        self.generate_amino_acids();
        self.mutate();
        self.recombine();
        self.recombine_in_protocells();
        self.drift();
        self.update_protocells();
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
