//! Molecules: small molecules and amino acid chains, with their constructors and queries.

use super::atom::{Atom, CARBON, HYDROGEN, NITROGEN, OXYGEN};
use super::position::Position;
use super::sequence::random_sequence;
use crate::events::ChainId;
use rand::Rng;

/// How likely a chain is to degrade (not used yet).
const DEGRADATION_RATE: f64 = 0.05;
/// Minimum energy for a chain to take part in reactions (not used yet).
const REACTION_THRESHOLD: f64 = 0.7;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A small molecule such as H2O or CH4.
    Simple(&'static str),
    /// A polypeptide, as a sequence of single-letter amino acid codes.
    AminoAcid(String),
    /// A fatty molecule with a water-loving head and an oily tail. Lipids cling
    /// together, and where they gather a droplet forms more easily.
    Lipid,
    /// One free building block of RNA: A, C, G or U.
    Nucleotide(char),
    /// A chain of nucleotides, written as its bases (for example "GGUACA").
    Strand(String),
}

/// A molecule: a name (or sequence), an energy, its atoms and a position in the soup.
#[derive(Clone, Debug, PartialEq)]
pub struct Molecule {
    /// Which chain this is in the event log. Only the soup hands out real ids;
    /// constructors leave it 0, and it means nothing for small molecules.
    pub id: ChainId,
    pub kind: Kind,
    pub energy: f64,
    pub atoms: Vec<Atom>,
    pub position: Position,
}

impl Molecule {
    pub fn new(name: &'static str, energy: f64, atoms: Vec<Atom>) -> Self {
        Self { id: 0, kind: Kind::Simple(name), energy, atoms, position: Position::default() }
    }

    /// A molecule of the given kind with no position yet.
    pub(super) fn from_kind(kind: Kind, energy: f64, atoms: Vec<Atom>) -> Self {
        Self { id: 0, kind, energy, atoms, position: Position::default() }
    }

    pub fn water() -> Self {
        Self::new("H2O", 1.0, vec![HYDROGEN, HYDROGEN, OXYGEN])
    }

    pub fn methane() -> Self {
        Self::new("CH4", 1.5, vec![CARBON, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN])
    }

    /// A chain with the given sequence. Each residue gets a simple C, N, O, H backbone.
    pub fn amino_acid(sequence: String, energy: f64) -> Self {
        let atoms = sequence.chars().flat_map(|_| [CARBON, NITROGEN, OXYGEN, HYDROGEN]).collect();
        Self { id: 0, kind: Kind::AminoAcid(sequence), energy, atoms, position: Position::default() }
    }

    /// A random chain of `min_len..=max_len` residues, with energy 1 + 0.1 per residue.
    pub fn random_chain(rng: &mut impl Rng, min_len: usize, max_len: usize) -> Self {
        let len = rng.random_range(min_len..=max_len);
        let sequence = random_sequence(rng, len);
        Self::amino_acid(sequence, 1.0 + len as f64 * 0.1)
    }

    /// "H2O", "CH4", or "AA<length>" for a chain.
    pub fn name(&self) -> String {
        match &self.kind {
            Kind::Simple(name) => (*name).to_string(),
            Kind::AminoAcid(seq) => format!("AA{}", seq.len()),
            Kind::Lipid => "Lipid".to_string(),
            Kind::Nucleotide(_) => "Nucleotide".to_string(),
            Kind::Strand(seq) => format!("RNA{}", seq.len()),
        }
    }

    pub fn sequence(&self) -> Option<&str> {
        match &self.kind {
            Kind::AminoAcid(seq) => Some(seq),
            _ => None,
        }
    }

    pub fn is_amino_acid(&self) -> bool {
        matches!(self.kind, Kind::AminoAcid(_))
    }

    /// Atom counts by element, in order of first appearance.
    pub fn composition(&self) -> Vec<(&'static str, usize)> {
        let mut counts: Vec<(&'static str, usize)> = Vec::new();
        for atom in &self.atoms {
            match counts.iter_mut().find(|(el, _)| *el == atom.element) {
                Some((_, n)) => *n += 1,
                None => counts.push((atom.element, 1)),
            }
        }
        counts
    }

    /// A short description of the shape of known small molecules.
    pub fn structure(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Simple("H2O") => Some("H-O-H (Bent)"),
            Kind::Simple("CH4") => Some("Tetrahedral"),
            Kind::Simple("CH3OH") => Some("H3C-OH (Methyl Alcohol)"),
            _ => None,
        }
    }


    pub(super) fn with_sequence(&self, sequence: String, energy: f64) -> Self {
        Self { position: self.position, ..Self::amino_acid(sequence, energy) }
    }

    pub(super) fn seq_or_empty(&self) -> &str {
        self.sequence().unwrap_or("")
    }

    /// Whether both chains have enough energy to react (not used yet).
    pub fn can_react_with(&self, other: &Molecule) -> bool {
        self.energy > REACTION_THRESHOLD && other.energy > REACTION_THRESHOLD
    }

    /// Longer chains degrade more easily (not used yet).
    pub fn degradation_probability(&self) -> f64 {
        DEGRADATION_RATE * (1.0 + self.seq_or_empty().len() as f64 * 0.05)
    }
}
