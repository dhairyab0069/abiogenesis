//! The chemistry of stage 3: lipids, nucleotides and RNA-like strands, and how
//! fit each kind of molecule is.
//!
//! Design choices:
//! - **Fitness is a number from 0 to 1 for every molecule.** It is one plain
//!   idea per kind, so a visitor can follow it: a chain is fit when it is a mix
//!   of oily and water-loving residues, a strand when its bases are balanced and
//!   it carries the catalytic motif.
//! - **The strands are RNA-like, with four letters (A, C, G, U).** Copying a
//!   strand builds its complement (A pairs with U, C with G), the way real RNA
//!   copies itself: one round gives the mirror image, a second gives the original.
//! - **One made-up catalytic motif.** A strand containing [`MOTIF`] copies itself
//!   faster. Real ribozymes are far richer; one short motif is enough to show
//!   autocatalysis and selection without any biochemistry lessons.
//! - **No simulation code here.** These are queries on one molecule, so they are
//!   easy to test. The soup decides how often things happen.

use super::atom::{CARBON, HYDROGEN, NITROGEN, OXYGEN, PHOSPHORUS};
use super::molecule::{Kind, Molecule};

/// The four RNA bases.
pub const BASES: [char; 4] = ['A', 'C', 'G', 'U'];
/// A strand that contains this stretch is a catalyst: it copies itself faster.
pub const MOTIF: &str = "GGU";
/// Residues that avoid water.
const HYDROPHOBIC: &str = "AVILMFWPGC";
/// Strands shorter than this are too flimsy to count as fit.
const STABLE_LENGTH: f64 = 6.0;

/// The base that pairs with `base`. Anything else comes back unchanged.
pub fn complement(base: char) -> char {
    match base {
        'A' => 'U',
        'U' => 'A',
        'C' => 'G',
        'G' => 'C',
        other => other,
    }
}

/// The strand a template builds: each base replaced by its partner.
pub fn complement_strand(sequence: &str) -> String {
    sequence.chars().map(complement).collect()
}

/// 1 when `x` is exactly half, falling to 0 at either extreme.
fn balance(x: f64) -> f64 {
    1.0 - (x - 0.5).abs() * 2.0
}

impl Molecule {
    pub fn lipid() -> Self {
        let atoms = std::iter::repeat_n(CARBON, 16).chain([OXYGEN, OXYGEN]).chain(std::iter::repeat_n(HYDROGEN, 32)).collect();
        Self::from_kind(Kind::Lipid, 1.2, atoms)
    }

    /// A free nucleotide. `base` should be one of [`BASES`].
    pub fn nucleotide(base: char) -> Self {
        Self::from_kind(Kind::Nucleotide(base), 1.0, vec![CARBON, NITROGEN, OXYGEN, PHOSPHORUS])
    }

    /// A strand with the given bases, with energy 1 + 0.1 per base like a chain.
    pub fn strand(sequence: String) -> Self {
        let atoms = sequence.chars().flat_map(|_| [CARBON, NITROGEN, OXYGEN, PHOSPHORUS]).collect();
        let energy = 1.0 + sequence.len() as f64 * 0.1;
        Self::from_kind(Kind::Strand(sequence), energy, atoms)
    }

    pub fn is_lipid(&self) -> bool {
        matches!(self.kind, Kind::Lipid)
    }

    pub fn is_nucleotide(&self) -> bool {
        matches!(self.kind, Kind::Nucleotide(_))
    }

    pub fn is_strand(&self) -> bool {
        matches!(self.kind, Kind::Strand(_))
    }

    /// The bases of a strand.
    pub fn strand_sequence(&self) -> Option<&str> {
        match &self.kind {
            Kind::Strand(seq) => Some(seq),
            _ => None,
        }
    }

    /// The letter of a free nucleotide.
    pub fn base(&self) -> Option<char> {
        match self.kind {
            Kind::Nucleotide(b) => Some(b),
            _ => None,
        }
    }

    /// The share of a strand's bases that are G or C (they bond more strongly).
    pub fn gc_fraction(&self) -> Option<f64> {
        let seq = self.strand_sequence().filter(|s| !s.is_empty())?;
        Some(seq.chars().filter(|c| matches!(c, 'G' | 'C')).count() as f64 / seq.len() as f64)
    }

    /// Whether this is a strand that carries the catalytic [`MOTIF`].
    pub fn is_catalyst(&self) -> bool {
        self.strand_sequence().is_some_and(|s| s.contains(MOTIF))
    }

    /// How well this molecule does in the soup, from 0 (hopeless) to 1 (ideal).
    ///
    /// - small molecules and free nucleotides: 0.5, they are raw material;
    /// - lipids: 0.8;
    /// - chains: 1 when half the residues avoid water, less when lopsided;
    /// - strands: balanced G and C (strong enough to last, loose enough to copy)
    ///   and long enough to hold together, plus 0.2 for the catalytic motif.
    pub fn fitness(&self) -> f64 {
        match &self.kind {
            Kind::Simple(_) | Kind::Nucleotide(_) => 0.5,
            Kind::Lipid => 0.8,
            Kind::AminoAcid(seq) if seq.is_empty() => 0.0,
            Kind::AminoAcid(seq) => balance(seq.chars().filter(|c| HYDROPHOBIC.contains(*c)).count() as f64 / seq.len() as f64),
            Kind::Strand(seq) => {
                let Some(gc) = self.gc_fraction() else { return 0.0 };
                let length = (seq.len() as f64 / STABLE_LENGTH).min(1.0);
                let motif = if self.is_catalyst() { 0.2 } else { 0.0 };
                (0.8 * balance(gc) * length + motif).clamp(0.0, 1.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complement_pairs_a_with_u_and_c_with_g() {
        assert_eq!(complement_strand("AUCG"), "UAGC");
        assert_eq!(complement_strand(&complement_strand("GGUACA")), "GGUACA");
        assert_eq!(complement('X'), 'X');
    }

    #[test]
    fn constructors_build_the_right_kind() {
        assert!(Molecule::lipid().is_lipid());
        assert_eq!(Molecule::nucleotide('G').base(), Some('G'));
        let s = Molecule::strand("GGUA".into());
        assert!(s.is_strand() && s.strand_sequence() == Some("GGUA") && s.name() == "RNA4");
        assert_eq!(s.atoms.len(), 16);
        assert!(!s.is_amino_acid() && s.sequence().is_none());
    }

    #[test]
    fn gc_fraction_counts_g_and_c() {
        assert_eq!(Molecule::strand("GCAU".into()).gc_fraction(), Some(0.5));
        assert_eq!(Molecule::strand("AAAA".into()).gc_fraction(), Some(0.0));
        assert_eq!(Molecule::lipid().gc_fraction(), None);
    }

    #[test]
    fn fitness_is_always_between_zero_and_one() {
        let all = [
            Molecule::water(),
            Molecule::lipid(),
            Molecule::nucleotide('A'),
            Molecule::strand(String::new()),
            Molecule::strand("GGGGGGGGGGGG".into()),
            Molecule::strand("GGUACAGCAUCG".into()),
            Molecule::amino_acid("AAAA".into(), 1.0),
            Molecule::amino_acid("AVKD".into(), 1.0),
            Molecule::amino_acid(String::new(), 1.0),
        ];
        for m in all {
            assert!((0.0..=1.0).contains(&m.fitness()), "{} has fitness {}", m.name(), m.fitness());
        }
    }

    #[test]
    fn balanced_strands_beat_lopsided_ones() {
        let balanced = Molecule::strand("GAUCAUGC".into());
        let all_gc = Molecule::strand("GGCCGGCC".into());
        let no_gc = Molecule::strand("AUAUAUAU".into());
        assert!(balanced.fitness() > all_gc.fitness() && balanced.fitness() > no_gc.fitness());
    }

    #[test]
    fn short_strands_are_less_fit_than_long_ones() {
        assert!(Molecule::strand("GA".into()).fitness() < Molecule::strand("GAUCAUGC".into()).fitness());
    }

    #[test]
    fn the_catalytic_motif_raises_fitness_and_marks_a_catalyst() {
        // Same letters, same balance: only the motif differs.
        let with = Molecule::strand("GGUCAUAC".into());
        let without = Molecule::strand("GAUCGUAC".into());
        assert_eq!(with.gc_fraction(), without.gc_fraction());
        assert!(with.is_catalyst() && !without.is_catalyst());
        assert!(with.fitness() > without.fitness());
    }

    #[test]
    fn chains_are_fittest_when_half_their_residues_avoid_water() {
        let mixed = Molecule::amino_acid("AVKD".into(), 1.0);
        let oily = Molecule::amino_acid("AVIL".into(), 1.0);
        assert!((mixed.fitness() - 1.0).abs() < 1e-9 && oily.fitness() < 0.1);
    }
}
