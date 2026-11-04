//! Mutation: chains occasionally change by substitution, insertion or deletion.

use super::PrimordialSoup;
use super::protocell::MUTATION_BOOST;
use crate::events::ChainEvent;
use rand::Rng;

/// Probability per step that a chain mutates.
const MUTATION_RATE: f64 = 0.01;

impl PrimordialSoup {
    /// Each chain has a small chance to mutate by substitution, insertion or deletion.
    /// The child replaces its parent in the soup, but is logged as a new chain
    /// linked to the parent. A mutation that makes no edit (deleting from a
    /// one-residue chain) leaves the chain and the log untouched.
    pub(super) fn mutate(&mut self) {
        let rng = &mut self.rng;
        for m in &mut self.molecules {
            let sheltered = self.protocells.iter().any(|p| p.contains(m.position.x, m.position.y));
            let rate = if sheltered { MUTATION_RATE * MUTATION_BOOST } else { MUTATION_RATE };
            if m.is_amino_acid() && rng.random::<f64>() < rate {
                let edit = match rng.random_range(0..3) {
                    0 => m.mutate_substitution(rng),
                    1 => m.mutate_insertion(rng),
                    _ => m.mutate_deletion(rng),
                };
                if let Some((mut child, mutation)) = edit {
                    child.id = self.events.new_id();
                    self.events.record(
                        self.step,
                        ChainEvent::Mutated {
                            id: child.id,
                            parent: m.id,
                            mutation,
                            sequence: child.sequence().unwrap_or("").to_string(),
                        },
                    );
                    *m = child;
                }
            }
        }
    }
}
