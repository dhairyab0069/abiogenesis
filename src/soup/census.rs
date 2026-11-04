//! Read-only summaries of the population.

use super::{PrimordialSoup, TARGET_PATTERN};

impl PrimordialSoup {
    pub fn amino_acid_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_amino_acid()).count()
    }

    /// The sequences that contain the target pattern.
    pub(super) fn target_matches(&self) -> Vec<String> {
        self.molecules
            .iter()
            .filter_map(|m| m.sequence())
            .filter(|s| s.contains(TARGET_PATTERN))
            .map(str::to_string)
            .collect()
    }

    /// Molecule counts by name, sorted by name.
    pub fn counts(&self) -> Vec<(String, usize)> {
        let mut counts = std::collections::BTreeMap::new();
        for m in &self.molecules {
            *counts.entry(m.name()).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }
}
