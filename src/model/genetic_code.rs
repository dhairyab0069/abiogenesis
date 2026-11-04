//! The genetic code: how an RNA strand spells a protein.
//!
//! Design choices:
//! - **The real code, not a toy one.** It is the standard table of 64 codons,
//!   built from the usual 64-letter string, so what the visitor sees matches a
//!   textbook. Making up a code would be easy, but then nothing here could be
//!   checked against anything.
//! - **A gene starts at the first AUG and stops at a stop codon.** AUG is both
//!   the start signal and the amino acid M. A strand that ends before a stop
//!   still counts: the protein is whatever was read so far.
//! - **Too-short results are not proteins.** A lone M tells us nothing, so a
//!   gene needs at least two amino acids.
//! - **Pure functions only.** The soup decides when a strand is read; this
//!   module only says what it says. The animated view uses the same code.

/// The order the table is laid out in.
const ORDER: [char; 4] = ['U', 'C', 'A', 'G'];
/// One letter per codon, in `ORDER` (first base slowest). `*` is a stop.
const TABLE: &str = "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG";
/// The start codon.
pub const START_CODON: &str = "AUG";
/// The longest strand in the soup, which is the longest gene a strand can hold.
pub const MAX_PROTEIN_GENE_BASES: usize = 24;
/// The shortest protein that counts: the start M plus one more.
pub const MIN_PROTEIN: usize = 2;

/// What a codon means.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Codon {
    Amino(char),
    Stop,
}

/// The meaning of a three-letter codon of A, C, G and U, or `None` if it is not one.
pub fn read_codon(codon: &str) -> Option<Codon> {
    let letters: Vec<char> = codon.chars().collect();
    if letters.len() != 3 {
        return None;
    }
    let mut index = 0;
    for c in letters {
        index = index * 4 + ORDER.iter().position(|&b| b == c)?;
    }
    Some(match TABLE.as_bytes()[index] as char {
        '*' => Codon::Stop,
        a => Codon::Amino(a),
    })
}

/// How a strand is read: where the gene starts and each codon with its meaning,
/// ending with the stop codon if the strand has one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingFrame {
    /// Index of the first base of AUG.
    pub start: usize,
    /// The codons read, in order. The last is a stop codon when the gene is closed.
    pub codons: Vec<(String, Codon)>,
}

impl ReadingFrame {
    /// The protein this frame makes: every amino acid before the stop.
    pub fn protein(&self) -> String {
        self.codons
            .iter()
            .filter_map(|(_, c)| {
                if let Codon::Amino(a) = c {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Whether the gene ends on a stop codon (rather than running off the strand).
    pub fn is_closed(&self) -> bool {
        matches!(self.codons.last(), Some((_, Codon::Stop)))
    }
}

/// Finds the first AUG in `rna` and reads codons from there to the first stop codon
/// or the end of the strand. `None` when there is no AUG.
pub fn reading_frame(rna: &str) -> Option<ReadingFrame> {
    let start = rna.find(START_CODON)?;
    let mut codons = Vec::new();
    for chunk in rna.as_bytes()[start..].as_chunks::<3>().0 {
        let text = std::str::from_utf8(chunk).ok()?;
        let meaning = read_codon(text)?;
        codons.push((text.to_string(), meaning));
        if meaning == Codon::Stop {
            break;
        }
    }
    Some(ReadingFrame { start, codons })
}

/// The protein an RNA strand codes for, if it carries a gene of at least
/// [`MIN_PROTEIN`] amino acids.
pub fn translate(rna: &str) -> Option<String> {
    let protein = reading_frame(rna)?.protein();
    (protein.len() >= MIN_PROTEIN).then_some(protein)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_start_codon_is_methionine() {
        assert_eq!(read_codon("AUG"), Some(Codon::Amino('M')));
    }

    #[test]
    fn known_codons_match_the_textbook() {
        for (codon, expected) in [
            ("UUU", 'F'),
            ("UUA", 'L'),
            ("GCU", 'A'),
            ("UGG", 'W'),
            ("GGG", 'G'),
            ("AAA", 'K'),
            ("GAU", 'D'),
            ("CGC", 'R'),
        ] {
            assert_eq!(read_codon(codon), Some(Codon::Amino(expected)), "{codon}");
        }
        for stop in ["UAA", "UAG", "UGA"] {
            assert_eq!(read_codon(stop), Some(Codon::Stop));
        }
    }

    #[test]
    fn the_table_has_three_stops_and_all_twenty_amino_acids() {
        let all: Vec<String> = ORDER
            .iter()
            .flat_map(|a| {
                ORDER
                    .iter()
                    .flat_map(move |b| ORDER.iter().map(move |c| format!("{a}{b}{c}")))
            })
            .collect();
        assert_eq!(all.len(), 64);
        let meanings: Vec<Codon> = all.iter().map(|c| read_codon(c).unwrap()).collect();
        assert_eq!(meanings.iter().filter(|m| **m == Codon::Stop).count(), 3);
        let mut aminos: Vec<char> = meanings
            .iter()
            .filter_map(|m| {
                if let Codon::Amino(a) = m {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect();
        aminos.sort_unstable();
        aminos.dedup();
        assert_eq!(aminos.len(), 20);
    }

    #[test]
    fn bad_codons_are_not_codons() {
        assert_eq!(read_codon("AU"), None);
        assert_eq!(read_codon("AUGG"), None);
        assert_eq!(read_codon("ATG"), None);
        assert_eq!(read_codon(""), None);
    }

    #[test]
    fn a_gene_starts_at_the_first_aug_and_stops_at_a_stop() {
        // GG | AUG UUU GGC | UAA | UU
        let frame = reading_frame("GGAUGUUUGGCUAAUU").unwrap();
        assert_eq!(frame.start, 2);
        assert_eq!(frame.protein(), "MFG");
        assert!(frame.is_closed());
        assert_eq!(translate("GGAUGUUUGGCUAAUU"), Some("MFG".into()));
    }

    #[test]
    fn a_strand_with_no_aug_codes_for_nothing() {
        assert_eq!(reading_frame("GGCUUUCCA"), None);
        assert_eq!(translate("GGCUUUCCA"), None);
    }

    #[test]
    fn a_gene_with_no_stop_runs_to_the_end_of_the_strand() {
        let frame = reading_frame("AUGGCUUU").unwrap();
        assert_eq!(frame.protein(), "MA");
        assert!(
            !frame.is_closed(),
            "the last two bases are not a whole codon"
        );
        assert_eq!(translate("AUGGCUUU"), Some("MA".into()));
    }

    #[test]
    fn one_amino_acid_is_not_a_protein() {
        assert_eq!(translate("AUGUAA"), None);
        assert_eq!(translate("AUG"), None);
        assert_eq!(translate("GAUGUAGC"), None);
    }

    #[test]
    fn an_early_stop_ends_the_protein() {
        assert_eq!(translate("AUGGCUUGAUUUGGC"), Some("MA".into()));
    }
}
