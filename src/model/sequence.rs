//! Random amino acid sequences.

use rand::Rng;

/// The 20 standard amino acids, by single-letter code.
pub const AMINO_ACIDS: [char; 20] = [
    'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I', 'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y',
    'V',
];

pub fn random_amino_acid(rng: &mut impl Rng) -> char {
    AMINO_ACIDS[rng.random_range(0..AMINO_ACIDS.len())]
}

pub fn random_sequence(rng: &mut impl Rng, len: usize) -> String {
    (0..len).map(|_| random_amino_acid(rng)).collect()
}

/// Random chains of 5 to 20 residues, for the chain generator screen.
pub fn random_chains(rng: &mut impl Rng, count: usize) -> Vec<String> {
    (0..count)
        .map(|_| {
            let len = rng.random_range(5..=20);
            random_sequence(rng, len)
        })
        .collect()
}
