//! Proteins in the soup: a strand that carries a gene now and then makes the
//! protein it spells, and a strand with a fit protein copies itself faster.
//!
//! Design choices:
//! - **The gene is read with the same code the animation shows.** `translate`
//!   decides whether a strand has a gene and what protein it spells, so the soup
//!   and the "Genes and Proteins" screen can never disagree.
//! - **Expression is a slow, random event.** Each step a strand with a gene has a
//!   small chance to make one protein, which appears as a new amino acid chain
//!   next to it (logged as `Translated`). Inside a protocell it happens 3 times
//!   as often: everything the ribosome needs is close by.
//! - **The chain cap holds.** At [`MAX_AMINO_ACIDS`] a new protein can still
//!   appear, but only by replacing the least fit chain, and only if it is
//!   fitter. So the soup never grows past its cap, and good proteins win places.
//! - **Proteins feed back.** The gene boost multiplies a strand's copying rate by
//!   up to 2, scaled by its protein's fitness. That closes the loop the stage is
//!   about: RNA makes protein, protein helps the RNA that made it.
//! - **Off with the chemistry.** `express_genes` is only called from the stage 3
//!   chemistry step, so plain soups draw no extra random numbers.

use super::{MAX_AMINO_ACIDS, PrimordialSoup};
use crate::events::ChainEvent;
use crate::model::{Molecule, Position, protein_fitness, translate};
use rand::Rng;

/// Chance per step that a strand with a gene makes a protein.
const EXPRESSION_RATE: f64 = 0.004;
/// Inside a protocell expression is this many times as likely.
pub(super) const SHELTER_EXPRESSION_BOOST: f64 = 3.0;
/// A strand whose protein has fitness 1 copies itself this much faster (1 + this).
const GENE_BOOST: f64 = 1.0;
/// How far from its strand a new protein appears.
const APPEAR_RADIUS: f64 = 14.0;

/// How much faster a strand copies itself because of its protein: 1 for a strand
/// with no gene, up to `1 + GENE_BOOST` for a gene whose protein has fitness 1.
pub fn gene_boost(sequence: &str) -> f64 {
    match translate(sequence) {
        Some(protein) => 1.0 + GENE_BOOST * protein_fitness(&protein),
        None => 1.0,
    }
}

/// Chance per step that a strand makes its protein: 0 without a gene.
pub(super) fn expression_rate(sequence: &str, sheltered: bool) -> f64 {
    if translate(sequence).is_none() {
        return 0.0;
    }
    if sheltered {
        EXPRESSION_RATE * SHELTER_EXPRESSION_BOOST
    } else {
        EXPRESSION_RATE
    }
}

impl PrimordialSoup {
    /// Proteins made by strands since the soup began.
    pub fn proteins_made(&self) -> u64 {
        self.proteins_made
    }

    /// Strands that carry a gene (they spell a protein of at least two amino acids).
    pub fn gene_count(&self) -> usize {
        self.molecules
            .iter()
            .filter_map(Molecule::strand_sequence)
            .filter(|s| translate(s).is_some())
            .count()
    }

    /// The average fitness of the proteins the strands in the soup would make, or 0.
    pub fn mean_protein_fitness(&self) -> f64 {
        let fits: Vec<f64> = self
            .molecules
            .iter()
            .filter_map(Molecule::strand_sequence)
            .filter_map(translate)
            .map(|p| protein_fitness(&p))
            .collect();
        if fits.is_empty() {
            0.0
        } else {
            fits.iter().sum::<f64>() / fits.len() as f64
        }
    }

    /// Each strand with a gene may make its protein this step.
    pub(super) fn express_genes(&mut self) {
        let genes: Vec<(Position, String)> = self
            .molecules
            .iter()
            .filter_map(|m| m.strand_sequence().map(|s| (m.position, s.to_string())))
            .filter(|(_, s)| translate(s).is_some())
            .collect();
        for (at, gene) in genes {
            let rate = expression_rate(&gene, self.sheltered(&at));
            if self.rng.random::<f64>() < rate {
                self.express(at, &gene);
            }
        }
    }

    /// Makes the protein of `gene` next to `at`, if there is room (or a worse chain to replace).
    fn express(&mut self, at: Position, gene: &str) {
        let Some(protein) = translate(gene) else {
            return;
        };
        let mut chain = Molecule::amino_acid(protein.clone(), 1.0 + protein.len() as f64 * 0.1);
        chain.position = Position::new(
            (at.x + self.rng.random_range(-APPEAR_RADIUS..APPEAR_RADIUS)).clamp(0.0, self.width),
            (at.y + self.rng.random_range(-APPEAR_RADIUS..APPEAR_RADIUS)).clamp(0.0, self.height),
        );
        let slot = if self.amino_acid_count() < MAX_AMINO_ACIDS {
            None
        } else {
            let worst = (0..self.molecules.len())
                .filter(|&i| self.molecules[i].is_amino_acid())
                .min_by(|&a, &b| {
                    self.molecules[a]
                        .fitness()
                        .total_cmp(&self.molecules[b].fitness())
                });
            match worst {
                Some(i) if self.molecules[i].fitness() < chain.fitness() => Some(i),
                _ => return,
            }
        };
        chain.id = self.events.new_id();
        self.events.record(
            self.step,
            ChainEvent::Translated {
                id: chain.id,
                gene: gene.to_string(),
                sequence: protein,
            },
        );
        self.proteins_made += 1;
        match slot {
            Some(i) => self.molecules[i] = chain,
            None => self.molecules.push(chain),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soup::Protocell;
    use crate::soup::chemistry::replication_rate;

    /// "AUG GCU UUC AAA GAU UAA" with a short lead: a gene for MAFKD.
    const GENE: &str = "GCAUGGCUUUCAAAGAUUAACG";

    fn at(mut m: Molecule, x: f64, y: f64) -> Molecule {
        m.position = Position::new(x, y);
        m
    }

    fn soup_with(molecules: Vec<Molecule>) -> PrimordialSoup {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 51);
        soup.molecules = molecules;
        soup.chemistry = true;
        soup
    }

    #[test]
    fn the_gene_boost_scales_with_protein_fitness() {
        assert_eq!(gene_boost("GGGGGGGG"), 1.0, "no AUG, no gene");
        assert_eq!(gene_boost("AUGUAA"), 1.0, "a lone M is not a protein");
        let boost = gene_boost(GENE);
        let fit = protein_fitness("MAFKD");
        assert!((boost - (1.0 + fit)).abs() < 1e-12 && boost > 1.5);
        // AUG UUU UUU UUU UAA: MFFF, all oily, so a poor protein.
        assert!(gene_boost("AUGUUUUUUUUUUAA") < boost);
        assert!(gene_boost(GENE) <= 1.0 + GENE_BOOST);
    }

    #[test]
    fn a_strand_with_a_fit_protein_replicates_faster() {
        assert!(replication_rate(GENE, false) > replication_rate("GCGGCUUUCAAAGAUUAACG", false));
        let plain = replication_rate("GCAGCUUUC", false);
        assert!((replication_rate(GENE, false) / plain - gene_boost(GENE)).abs() < 1e-9);
    }

    #[test]
    fn only_strands_with_genes_express_and_shelter_speeds_it_up() {
        assert_eq!(expression_rate("GGGGGG", true), 0.0);
        assert!(expression_rate(GENE, false) > 0.0);
        assert!(expression_rate(GENE, true) > expression_rate(GENE, false));
    }

    #[test]
    fn a_gene_makes_its_protein_next_to_the_strand() {
        let mut soup = soup_with(vec![at(Molecule::strand(GENE.into()), 200.0, 150.0)]);
        for _ in 0..10_000 {
            soup.express_genes();
            if soup.proteins_made() > 0 {
                break;
            }
        }
        assert!(soup.proteins_made() >= 1);
        let chain = soup
            .molecules
            .iter()
            .find(|m| m.is_amino_acid())
            .expect("a protein appeared");
        assert_eq!(chain.sequence(), Some("MAFKD"));
        assert!(chain.position.distance_to(&Position::new(200.0, 150.0)) < 25.0);
        let event = soup.events.creation_of(chain.id).expect("logged");
        assert!(
            matches!(&event.event, ChainEvent::Translated { gene, sequence, .. } if gene == GENE && sequence == "MAFKD")
        );
    }

    #[test]
    fn a_strand_without_a_gene_makes_nothing() {
        let mut soup = soup_with(vec![at(
            Molecule::strand("GGGCCCGGGCCC".into()),
            200.0,
            150.0,
        )]);
        for _ in 0..10_000 {
            soup.express_genes();
        }
        assert_eq!(soup.proteins_made(), 0);
        assert_eq!(soup.amino_acid_count(), 0);
        assert_eq!(soup.gene_count(), 0);
    }

    #[test]
    fn expression_never_passes_the_chain_cap() {
        let mut crowd: Vec<Molecule> = (0..MAX_AMINO_ACIDS)
            .map(|k| {
                at(
                    Molecule::amino_acid("AAAA".into(), 1.0),
                    10.0 + k as f64,
                    20.0,
                )
            })
            .collect();
        crowd.extend(
            (0..10).map(|k| at(Molecule::strand(GENE.into()), 50.0 + k as f64 * 20.0, 150.0)),
        );
        let mut soup = soup_with(crowd);
        for _ in 0..3_000 {
            soup.express_genes();
            assert_eq!(soup.amino_acid_count(), MAX_AMINO_ACIDS);
        }
        // AAAA is all oily (fitness near 0), so the fitter MAFKD replaces some of it.
        assert!(soup.proteins_made() > 0);
        assert!(soup.molecules.iter().any(|m| m.sequence() == Some("MAFKD")));
    }

    #[test]
    fn at_the_cap_a_worse_protein_does_not_replace_a_better_chain() {
        let mut crowd: Vec<Molecule> = (0..MAX_AMINO_ACIDS)
            .map(|k| {
                at(
                    Molecule::amino_acid("AVKD".into(), 1.0),
                    10.0 + k as f64,
                    20.0,
                )
            })
            .collect();
        crowd.push(at(Molecule::strand("AUGUUUUUUUUUUAA".into()), 200.0, 150.0));
        let mut soup = soup_with(crowd);
        for _ in 0..3_000 {
            soup.express_genes();
        }
        assert_eq!(soup.proteins_made(), 0);
    }

    #[test]
    fn sheltered_strands_make_more_proteins() {
        let run = |sheltered: bool| {
            let mut soup = soup_with(vec![at(Molecule::strand(GENE.into()), 200.0, 150.0)]);
            if sheltered {
                soup.protocells.push(Protocell {
                    id: 1,
                    x: 200.0,
                    y: 150.0,
                    radius: 40.0,
                    born_step: 0,
                });
            }
            for _ in 0..4_000 {
                soup.express_genes();
                soup.molecules.retain(|m| m.is_strand()); // keep the cap out of it
            }
            soup.proteins_made()
        };
        let (open, inside) = (run(false), run(true));
        assert!(inside > open, "{inside} sheltered vs {open} open");
    }

    #[test]
    fn counters_and_mean_fitness_describe_the_genes() {
        let soup = soup_with(vec![
            at(Molecule::strand(GENE.into()), 10.0, 10.0),
            at(Molecule::strand("CCCCCC".into()), 20.0, 10.0),
        ]);
        assert_eq!(soup.gene_count(), 1);
        assert!((soup.mean_protein_fitness() - protein_fitness("MAFKD")).abs() < 1e-12);
        assert_eq!(soup_with(vec![]).mean_protein_fitness(), 0.0);
    }

    #[test]
    fn seeded_soups_with_chemistry_are_deterministic() {
        let run = || {
            let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 77);
            soup.add_prebiotic_chemistry();
            for _ in 0..1_500 {
                soup.simulate_step();
            }
            (
                soup.proteins_made(),
                soup.strand_count(),
                soup.events.len(),
                soup.history.samples().len(),
            )
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn a_long_standard_run_makes_proteins_and_keeps_every_cap() {
        let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 4);
        soup.add_small_molecules();
        soup.add_prebiotic_chemistry();
        soup.set_environment(crate::environment::Environment::preset(
            crate::environment::Preset::Both,
            300.0,
            7.0,
        ));
        for _ in 0..10_000 {
            soup.simulate_step();
            assert!(
                soup.amino_acid_count()
                    <= MAX_AMINO_ACIDS + super::super::recombination::MAX_COMBINATIONS_PER_STEP
            );
        }
        eprintln!(
            "proteins {} genes {} strands {}",
            soup.proteins_made(),
            soup.gene_count(),
            soup.strand_count()
        );
        assert!(
            soup.proteins_made() > 0,
            "genes should appear and be read in a long run"
        );
    }
}
