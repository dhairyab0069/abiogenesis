//! Chain-editing operations: joining, splitting, mutation and recombination.

use super::molecule::Molecule;
use super::sequence::random_amino_acid;
use crate::events::Mutation;
use rand::Rng;

impl Molecule {
    /// Joins two chains end to end. Energy is summed, minus 0.5 for the reaction.
    pub fn combine(a: &Molecule, b: &Molecule) -> Molecule {
        let sequence = format!("{}{}", a.seq_or_empty(), b.seq_or_empty());
        Self::amino_acid(sequence, a.energy + b.energy - 0.5)
    }

    /// A shorter chain split off at a random point, keeping 60% of the energy.
    /// None if the chain is a single residue (not used yet).
    pub fn degrade(&self, rng: &mut impl Rng) -> Option<Molecule> {
        let seq = self.seq_or_empty();
        if seq.len() <= 1 {
            return None;
        }
        let split = rng.random_range(1..seq.len());
        Some(self.with_sequence(seq[..split].to_string(), self.energy * 0.6))
    }


    /// Replaces one random residue with a random amino acid. Returns the child and
    /// the edit, or None (with no randomness used) for an empty chain.
    ///
    /// Edge cases return `None` rather than a clone with a fake edit: "nothing
    /// happened" is then impossible to mistake for a mutation in the event log.
    pub fn mutate_substitution(&self, rng: &mut impl Rng) -> Option<(Molecule, Mutation)> {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        if chars.is_empty() {
            return None;
        }
        let pos = rng.random_range(0..chars.len());
        let from = chars[pos];
        let to = random_amino_acid(rng);
        chars[pos] = to;
        let child = self.with_sequence(chars.into_iter().collect(), self.energy);
        Some((child, Mutation::Substitution { index: pos, from, to }))
    }

    /// Inserts a random amino acid at a random position. Always makes an edit.
    pub fn mutate_insertion(&self, rng: &mut impl Rng) -> Option<(Molecule, Mutation)> {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        let pos = rng.random_range(0..=chars.len());
        let residue = random_amino_acid(rng);
        chars.insert(pos, residue);
        let child = self.with_sequence(chars.into_iter().collect(), self.energy);
        Some((child, Mutation::Insertion { index: pos, residue }))
    }

    /// Deletes one random residue. None (no randomness used) for a chain of one
    /// residue or fewer, which is never shortened.
    pub fn mutate_deletion(&self, rng: &mut impl Rng) -> Option<(Molecule, Mutation)> {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        if chars.len() <= 1 {
            return None;
        }
        let pos = rng.random_range(0..chars.len());
        let residue = chars.remove(pos);
        let child = self.with_sequence(chars.into_iter().collect(), self.energy);
        Some((child, Mutation::Deletion { index: pos, residue }))
    }

    /// Recombination: the start of this chain joined to the end of `other`, with
    /// the parents' average energy. Also returns the cut points `(in self, in
    /// other)`. None (no randomness used) if either parent has no residues.
    pub fn crossover(&self, other: &Molecule, rng: &mut impl Rng) -> Option<(Molecule, (usize, usize))> {
        let (a, b) = (self.seq_or_empty(), other.seq_or_empty());
        if a.is_empty() || b.is_empty() {
            return None;
        }
        let p1 = rng.random_range(0..a.len());
        let p2 = rng.random_range(0..b.len());
        let sequence = format!("{}{}", &a[..p1], &b[p2..]);
        Some((Self::amino_acid(sequence, (self.energy + other.energy) / 2.0), (p1, p2)))
    }
}
