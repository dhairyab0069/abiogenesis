//! A recorder of how the soup changes over time, for the charts.
//!
//! Design choices:
//! - **Sampled, not logged every step.** One sample every [`SAMPLE_EVERY`] steps
//!   keeps the cost negligible and the charts readable (60 steps a second means
//!   two samples a second).
//! - **Bounded.** At most [`MAX_SAMPLES`] samples are kept; when it is full the
//!   oldest goes. A chart therefore shows the recent past, never grows without
//!   limit, and a soup left running all day uses the same memory as a new one.
//! - **Plain numbers.** A [`Sample`] is just numbers taken from the soup, and a
//!   [`Metric`] picks one of them. The chart code needs nothing else.
//! - **No randomness.** Recording never draws a random number, so it cannot
//!   change a seeded run.

use super::PrimordialSoup;
use crate::model::Molecule;
use std::collections::{HashSet, VecDeque};

/// A sample is taken on every step that is a multiple of this.
pub const SAMPLE_EVERY: u64 = 30;
/// The most samples kept.
pub const MAX_SAMPLES: usize = 400;

/// What the soup looked like at one moment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sample {
    pub step: u64,
    pub strands: usize,
    pub catalysts: usize,
    pub mean_strand_fitness: f64,
    pub proteins_made: u64,
    /// How many different strand sequences there are.
    pub diversity: usize,
    pub temperature: f64,
    pub wetness: f64,
    pub protocells: usize,
    /// Cells of stage 5 (all kinds), eukaryotes among them, and colonies.
    pub cells: usize,
    pub eukaryotes: usize,
    pub colonies: usize,
}

/// One chart: which number of a [`Sample`] to follow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Strands,
    Catalysts,
    StrandFitness,
    ProteinsMade,
    Diversity,
    Temperature,
    Wetness,
    Protocells,
    Cells,
    Eukaryotes,
    Colonies,
}

impl Metric {
    pub const ALL: [Metric; 11] = [
        Metric::Strands,
        Metric::Catalysts,
        Metric::StrandFitness,
        Metric::ProteinsMade,
        Metric::Diversity,
        Metric::Temperature,
        Metric::Wetness,
        Metric::Protocells,
        Metric::Cells,
        Metric::Eukaryotes,
        Metric::Colonies,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Metric::Strands => "RNA strands",
            Metric::Catalysts => "Catalysts",
            Metric::StrandFitness => "Strand fitness",
            Metric::ProteinsMade => "Proteins made",
            Metric::Diversity => "Diversity (distinct strands)",
            Metric::Temperature => "Temperature (K)",
            Metric::Wetness => "Pool level",
            Metric::Protocells => "Protocells",
            Metric::Cells => "Cells",
            Metric::Eukaryotes => "Eukaryotes",
            Metric::Colonies => "Colonies",
        }
    }

    pub fn value(self, s: &Sample) -> f64 {
        match self {
            Metric::Strands => s.strands as f64,
            Metric::Catalysts => s.catalysts as f64,
            Metric::StrandFitness => s.mean_strand_fitness,
            Metric::ProteinsMade => s.proteins_made as f64,
            Metric::Diversity => s.diversity as f64,
            Metric::Temperature => s.temperature,
            Metric::Wetness => s.wetness,
            Metric::Protocells => s.protocells as f64,
            Metric::Cells => s.cells as f64,
            Metric::Eukaryotes => s.eukaryotes as f64,
            Metric::Colonies => s.colonies as f64,
        }
    }
}

/// The recorded samples, oldest first.
#[derive(Clone, Debug, Default)]
pub struct History {
    samples: VecDeque<Sample>,
}

impl History {
    /// Whether a sample is due on `step`.
    pub fn is_due(step: u64) -> bool {
        step > 0 && step.is_multiple_of(SAMPLE_EVERY)
    }

    /// Adds a sample, dropping the oldest when full.
    pub fn push(&mut self, sample: Sample) {
        if self.samples.len() >= MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    pub fn samples(&self) -> &VecDeque<Sample> {
        &self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// The values of one metric, oldest first.
    pub fn series(&self, metric: Metric) -> Vec<f64> {
        self.samples.iter().map(|s| metric.value(s)).collect()
    }
}

impl PrimordialSoup {
    /// What the soup looks like right now.
    pub fn sample(&self) -> Sample {
        let strands: Vec<&str> = self.molecules.iter().filter_map(Molecule::strand_sequence).collect();
        Sample {
            step: self.step,
            strands: strands.len(),
            catalysts: self.catalyst_count(),
            mean_strand_fitness: self.mean_strand_fitness(),
            proteins_made: self.proteins_made,
            diversity: strands.iter().collect::<HashSet<_>>().len(),
            temperature: self.temperature,
            wetness: self.wetness,
            protocells: self.protocells.len(),
            cells: self.cell_count(),
            eukaryotes: self.eukaryote_count(),
            colonies: self.colony_count(),
        }
    }

    /// Records a sample when one is due. Called at the end of every step.
    pub(super) fn record_history(&mut self) {
        if History::is_due(self.step) {
            let sample = self.sample();
            self.history.push(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Position;

    fn strand(seq: &str, x: f64) -> Molecule {
        let mut m = Molecule::strand(seq.into());
        m.position = Position::new(x, 10.0);
        m
    }

    #[test]
    fn samples_are_taken_every_n_steps() {
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 300.0, 7.0, 1);
        for _ in 0..(SAMPLE_EVERY * 3 + 5) {
            soup.simulate_step();
        }
        let steps: Vec<u64> = soup.history.samples().iter().map(|s| s.step).collect();
        assert_eq!(steps, vec![SAMPLE_EVERY, SAMPLE_EVERY * 2, SAMPLE_EVERY * 3]);
        assert!(!History::is_due(0) && History::is_due(SAMPLE_EVERY) && !History::is_due(SAMPLE_EVERY + 1));
    }

    #[test]
    fn a_sample_describes_the_strands() {
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 300.0, 7.0, 2);
        soup.molecules = vec![strand("GGUCAUAC", 10.0), strand("GGUCAUAC", 20.0), strand("AAAAAA", 30.0)];
        let s = soup.sample();
        assert_eq!((s.strands, s.catalysts, s.diversity, s.protocells), (3, 2, 2, 0));
        let mean = soup.molecules.iter().map(Molecule::fitness).sum::<f64>() / 3.0;
        assert!((s.mean_strand_fitness - mean).abs() < 1e-12);
        assert_eq!(s.temperature, 300.0);
    }

    #[test]
    fn the_history_is_bounded_and_drops_the_oldest() {
        let mut h = History::default();
        assert!(h.is_empty());
        for step in 0..(MAX_SAMPLES as u64 + 25) {
            h.push(Sample { step, ..Sample::default() });
        }
        assert_eq!(h.len(), MAX_SAMPLES);
        assert_eq!(h.samples().front().map(|s| s.step), Some(25));
        assert_eq!(h.samples().back().map(|s| s.step), Some(MAX_SAMPLES as u64 + 24));
    }

    #[test]
    fn series_pick_one_metric() {
        let mut h = History::default();
        for k in 0..4 {
            h.push(Sample { strands: k, temperature: 300.0 + k as f64, ..Sample::default() });
        }
        assert_eq!(h.series(Metric::Strands), vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(h.series(Metric::Temperature), vec![300.0, 301.0, 302.0, 303.0]);
        for m in Metric::ALL {
            assert_eq!(h.series(m).len(), 4);
            assert!(!m.label().is_empty());
        }
    }

    #[test]
    fn recording_does_not_change_a_seeded_run() {
        let run = |record: bool| {
            let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 300.0, 7.0, 9);
            soup.add_prebiotic_chemistry();
            for _ in 0..600 {
                soup.simulate_step();
                if !record {
                    soup.history = History::default();
                }
            }
            (soup.molecules.len(), soup.strand_count(), soup.events.len())
        };
        assert_eq!(run(true), run(false));
    }
}
