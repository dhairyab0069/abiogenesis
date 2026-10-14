//! Atoms, molecules and amino acid chains.

use rand::Rng;

/// The 20 standard amino acids, by single-letter code.
pub const AMINO_ACIDS: [char; 20] = [
    'A', 'R', 'N', 'D', 'C', 'Q', 'E', 'G', 'H', 'I', 'L', 'K', 'M', 'F', 'P', 'S', 'T', 'W', 'Y', 'V',
];

/// How likely a chain is to degrade (not used yet).
const DEGRADATION_RATE: f64 = 0.05;
/// Minimum energy for a chain to take part in reactions (not used yet).
const REACTION_THRESHOLD: f64 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Atom {
    pub element: &'static str,
    pub atomic_number: u8,
    pub atomic_mass: f64,
}

pub const HYDROGEN: Atom = Atom { element: "H", atomic_number: 1, atomic_mass: 1.008 };
pub const CARBON: Atom = Atom { element: "C", atomic_number: 6, atomic_mass: 12.011 };
pub const NITROGEN: Atom = Atom { element: "N", atomic_number: 7, atomic_mass: 14.007 };
pub const OXYGEN: Atom = Atom { element: "O", atomic_number: 8, atomic_mass: 15.999 };

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A small molecule such as H2O or CH4.
    Simple(&'static str),
    /// A polypeptide, as a sequence of single-letter amino acid codes.
    AminoAcid(String),
}

/// A molecule: a name (or sequence), an energy, its atoms and a position in the soup.
#[derive(Clone, Debug, PartialEq)]
pub struct Molecule {
    pub kind: Kind,
    pub energy: f64,
    pub atoms: Vec<Atom>,
    pub position: Position,
}

impl Molecule {
    pub fn new(name: &'static str, energy: f64, atoms: Vec<Atom>) -> Self {
        Self { kind: Kind::Simple(name), energy, atoms, position: Position::default() }
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
        Self { kind: Kind::AminoAcid(sequence), energy, atoms, position: Position::default() }
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
        }
    }

    pub fn sequence(&self) -> Option<&str> {
        match &self.kind {
            Kind::AminoAcid(seq) => Some(seq),
            Kind::Simple(_) => None,
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

    fn with_sequence(&self, sequence: String, energy: f64) -> Self {
        Self { position: self.position, ..Self::amino_acid(sequence, energy) }
    }

    fn seq_or_empty(&self) -> &str {
        self.sequence().unwrap_or("")
    }

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

    /// Whether both chains have enough energy to react (not used yet).
    pub fn can_react_with(&self, other: &Molecule) -> bool {
        self.energy > REACTION_THRESHOLD && other.energy > REACTION_THRESHOLD
    }

    /// Longer chains degrade more easily (not used yet).
    pub fn degradation_probability(&self) -> f64 {
        DEGRADATION_RATE * (1.0 + self.seq_or_empty().len() as f64 * 0.05)
    }

    /// Replaces one random residue with a random amino acid.
    pub fn mutate_substitution(&self, rng: &mut impl Rng) -> Molecule {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        if chars.is_empty() {
            return self.clone();
        }
        let pos = rng.random_range(0..chars.len());
        chars[pos] = random_amino_acid(rng);
        self.with_sequence(chars.into_iter().collect(), self.energy)
    }

    /// Inserts a random amino acid at a random position.
    pub fn mutate_insertion(&self, rng: &mut impl Rng) -> Molecule {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        let pos = rng.random_range(0..=chars.len());
        chars.insert(pos, random_amino_acid(rng));
        self.with_sequence(chars.into_iter().collect(), self.energy)
    }

    /// Deletes one random residue, unless the chain is a single residue.
    pub fn mutate_deletion(&self, rng: &mut impl Rng) -> Molecule {
        let mut chars: Vec<char> = self.seq_or_empty().chars().collect();
        if chars.len() <= 1 {
            return self.clone();
        }
        let pos = rng.random_range(0..chars.len());
        chars.remove(pos);
        self.with_sequence(chars.into_iter().collect(), self.energy)
    }

    /// Recombination: the start of this chain joined to the end of `other`, with
    /// the parents' average energy.
    pub fn crossover(&self, other: &Molecule, rng: &mut impl Rng) -> Molecule {
        let (a, b) = (self.seq_or_empty(), other.seq_or_empty());
        if a.is_empty() || b.is_empty() {
            return self.clone();
        }
        let p1 = rng.random_range(0..a.len());
        let p2 = rng.random_range(0..b.len());
        let sequence = format!("{}{}", &a[..p1], &b[p2..]);
        Self::amino_acid(sequence, (self.energy + other.energy) / 2.0)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng, rngs::StdRng};

    fn rng() -> StdRng {
        StdRng::seed_from_u64(7)
    }

    fn chain(seq: &str) -> Molecule {
        Molecule::amino_acid(seq.to_string(), 2.0)
    }

    #[test]
    fn small_molecules_have_expected_atoms() {
        assert_eq!(Molecule::water().composition(), vec![("H", 2), ("O", 1)]);
        assert_eq!(Molecule::methane().composition(), vec![("C", 1), ("H", 4)]);
        assert_eq!(Molecule::water().structure(), Some("H-O-H (Bent)"));
    }

    #[test]
    fn chain_has_four_backbone_atoms_per_residue() {
        let m = chain("MET");
        assert_eq!(m.name(), "AA3");
        assert_eq!(m.atoms.len(), 12);
        assert_eq!(m.composition(), vec![("C", 3), ("N", 3), ("O", 3), ("H", 3)]);
    }

    #[test]
    fn random_chain_respects_length_and_alphabet() {
        let mut r = rng();
        for _ in 0..200 {
            let m = Molecule::random_chain(&mut r, 1, 3);
            let seq = m.sequence().unwrap();
            assert!((1..=3).contains(&seq.len()));
            assert!(seq.chars().all(|c| AMINO_ACIDS.contains(&c)));
            assert!((m.energy - (1.0 + seq.len() as f64 * 0.1)).abs() < 1e-9);
        }
        for s in random_chains(&mut r, 50) {
            assert!((5..=20).contains(&s.len()));
        }
    }

    #[test]
    fn mutations_change_length_as_expected() {
        let mut r = rng();
        let m = chain("METHINKS");
        assert_eq!(m.mutate_substitution(&mut r).sequence().unwrap().len(), 8);
        assert_eq!(m.mutate_insertion(&mut r).sequence().unwrap().len(), 9);
        assert_eq!(m.mutate_deletion(&mut r).sequence().unwrap().len(), 7);
        assert_eq!(chain("M").mutate_deletion(&mut r).sequence(), Some("M"));
    }

    #[test]
    fn mutations_keep_position_and_energy() {
        let mut r = rng();
        let mut m = chain("AAAA");
        m.position = Position::new(3.0, 4.0);
        let mutated = m.mutate_insertion(&mut r);
        assert_eq!(mutated.position, m.position);
        assert_eq!(mutated.energy, m.energy);
    }

    #[test]
    fn crossover_combine_and_degrade() {
        let mut r = rng();
        let (a, b) = (chain("AAAA"), chain("CCCC"));
        let child = a.crossover(&b, &mut r);
        let seq = child.sequence().unwrap();
        assert!(seq.trim_start_matches('A').chars().all(|c| c == 'C'));
        assert_eq!(child.energy, 2.0);
        let joined = Molecule::combine(&a, &b);
        assert_eq!(joined.sequence(), Some("AAAACCCC"));
        assert_eq!(joined.energy, 3.5);
        let d = a.degrade(&mut r).unwrap();
        assert!(d.sequence().unwrap().len() < 4);
        assert!(chain("A").degrade(&mut r).is_none());
    }

    #[test]
    fn distance() {
        assert_eq!(Position::new(0.0, 0.0).distance_to(&Position::new(3.0, 4.0)), 5.0);
    }
}
