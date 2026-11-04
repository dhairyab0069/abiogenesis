//! Atoms, molecules and amino acid chains.

mod atom;
mod chemistry;
mod editing;
mod genetic_code;
mod molecule;
mod position;
mod sequence;
#[cfg(test)]
mod tests;

pub use atom::{Atom, CARBON, HYDROGEN, NITROGEN, OXYGEN, PHOSPHORUS};
pub use chemistry::{BASES, MOTIF, complement, complement_strand, protein_fitness};
pub use genetic_code::{
    Codon, MAX_PROTEIN_GENE_BASES, ReadingFrame, START_CODON, read_codon, reading_frame, translate,
};
pub use molecule::{Kind, Molecule};
pub use position::Position;
pub use sequence::{AMINO_ACIDS, random_amino_acid, random_chains, random_sequence};
