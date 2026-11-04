//! Creation of new chains: the seed population and ongoing generation.

use super::{MAX_AMINO_ACIDS, PrimordialSoup};
use crate::events::ChainEvent;
use crate::model::{Molecule, Position};
use rand::Rng;

pub(super) const INITIAL_AMINO_ACIDS: usize = 20;
/// Probability per step that a new chain is generated.
const AMINO_ACID_GENERATION_RATE: f64 = 0.005;

impl PrimordialSoup {
    pub fn add_small_molecules(&mut self) {
        for _ in 0..50 {
            for mut m in [Molecule::water(), Molecule::methane()] {
                m.position = self.random_position();
                self.add_molecule(m);
            }
        }
    }

    pub(super) fn random_position(&mut self) -> Position {
        Position::new(
            self.rng.random::<f64>() * self.width,
            self.rng.random::<f64>() * self.height,
        )
    }

    /// Adds a new random chain and logs it as generated. The one place chains
    /// appear from nothing, so seeding and later generation cannot drift apart.
    pub(super) fn spawn_chain(&mut self) {
        let mut chain = Molecule::random_chain(&mut self.rng, 1, 3);
        chain.position = self.random_position();
        chain.id = self.events.new_id();
        self.events.record(
            self.step,
            ChainEvent::Generated {
                id: chain.id,
                sequence: chain.sequence().unwrap_or("").to_string(),
            },
        );
        self.molecules.push(chain);
    }

    pub(super) fn generate_amino_acids(&mut self) {
        if self.amino_acid_count() < MAX_AMINO_ACIDS
            && self.rng.random::<f64>() < AMINO_ACID_GENERATION_RATE
        {
            self.spawn_chain();
        }
    }
}
