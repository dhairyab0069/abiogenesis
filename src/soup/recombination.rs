//! Recombination: pairs of chains occasionally cross over into a new chain.

use super::{MAX_AMINO_ACIDS, PrimordialSoup};
use crate::events::ChainEvent;
use crate::model::Position;
use rand::Rng;

/// Probability that a picked pair of chains recombines.
const COMBINATION_RATE: f64 = 0.01;

pub(super) const MAX_COMBINATIONS_PER_STEP: usize = 10;
/// Bounds the pair-picking loop, which could otherwise spin forever when the
/// soup has fewer than two chains.
const MAX_PAIR_ATTEMPTS: usize = 2_000;

impl PrimordialSoup {
    /// Picks random pairs from the current population and occasionally adds their
    /// crossover child at the pair's midpoint, while the soup is under the cap.
    pub(super) fn recombine(&mut self) {
        let snapshot_len = self.molecules.len();
        if snapshot_len < 2 {
            return;
        }
        let mut combinations = 0;
        for _ in 0..MAX_PAIR_ATTEMPTS {
            if combinations >= MAX_COMBINATIONS_PER_STEP || self.molecules.len() >= MAX_AMINO_ACIDS {
                break;
            }
            let i = self.rng.random_range(0..snapshot_len);
            let j = self.rng.random_range(0..snapshot_len);
            if i == j {
                continue;
            }
            let (a, b) = (&self.molecules[i], &self.molecules[j]);
            if a.is_amino_acid() && b.is_amino_acid() && self.rng.random::<f64>() < COMBINATION_RATE && self.cross(i, j) {
                combinations += 1;
            }
        }
    }

    /// Crosses chains `i` and `j`: adds the child at their midpoint and logs it.
    /// Returns false (and changes nothing) when the pair cannot be crossed.
    pub(super) fn cross(&mut self, i: usize, j: usize) -> bool {
        let (a, b) = (&self.molecules[i], &self.molecules[j]);
        // Chains that are empty cannot be crossed (never happens in a normal run);
        // skip them rather than add a duplicate of a parent.
        let Some((mut child, cut)) = a.crossover(b, &mut self.rng) else { return false };
        child.position = Position::new((a.position.x + b.position.x) / 2.0, (a.position.y + b.position.y) / 2.0);
        child.id = self.events.new_id();
        self.events.record(
            self.step,
            ChainEvent::Recombined {
                id: child.id,
                parents: (a.id, b.id),
                cut,
                sequence: child.sequence().unwrap_or("").to_string(),
            },
        );
        self.molecules.push(child);
        true
    }
}
