//! The "Genes and Proteins" scene as a pure timeline: no drawing, no egui.
//!
//! Design choices:
//! - **A timeline, not a physics simulation.** The scene is one looping story in
//!   seven phases: beads join into a strand, complementary bases dock under it,
//!   the pair twists into a helix, the helix unzips, a ribosome reads the strand
//!   into a protein, the protein folds, and a short rest. Every phase has a
//!   duration, and the time since the start of the loop says exactly where
//!   everything is. Drawing only turns a [`Snapshot`] into shapes.
//! - **Same functions as the simulation.** The copy is `complement_strand`, the
//!   reading frame is `reading_frame` and the protein is `translate`, the
//!   functions the soup uses. The scene can never show something the soup would
//!   not do.
//! - **DNA or RNA is only a spelling.** The toggle changes U to T on screen (and
//!   the pairs to A-T). The ribosome always reads messenger RNA, so in DNA mode
//!   the reading phase shows U again, with a note that the gene was transcribed.
//! - **Counts are fractional.** `joined: 2.4` means two beads are in place and the
//!   third is 40% of the way. That lets the drawing animate one bond at a time
//!   with no per-bead state.

use crate::model::{
    Codon, MAX_PROTEIN_GENE_BASES, ReadingFrame, complement_strand, protein_fitness, reading_frame,
    translate,
};
use rand::Rng;

/// Seconds a bead takes to join the strand.
const JOIN_TIME: f32 = 0.45;
/// Seconds a complementary base takes to dock.
const DOCK_TIME: f32 = 0.4;
const TWIST_TIME: f32 = 3.0;
const UNZIP_TIME_PER_BASE: f32 = 0.2;
const UNZIP_TIME_BASE: f32 = 1.0;
/// Seconds the ribosome takes to slide over one base before the start codon.
const SCAN_TIME: f32 = 0.22;
/// Seconds to read one codon.
const CODON_TIME: f32 = 1.1;
const FOLD_TIME: f32 = 3.0;
const REST_TIME: f32 = 2.5;
pub const MIN_SPEED: f32 = 0.25;
pub const MAX_SPEED: f32 = 4.0;
/// The gene shown at first: it spells MAFKD.
pub const DEFAULT_GENE: &str = "GCAUGGCUUUCAAAGAUUAACG";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    /// Free beads float in and join into a strand.
    Assemble,
    /// Complementary bases dock under each base.
    Pair,
    /// The pair twists into a double helix.
    Twist,
    /// The helix unzips into two strands.
    Unzip,
    /// A ribosome reads the strand and a chain of amino acids grows.
    Translate,
    /// The released chain folds into a compact coil.
    Fold,
    Rest,
}

impl Phase {
    pub const ALL: [Phase; 7] = [
        Phase::Assemble,
        Phase::Pair,
        Phase::Twist,
        Phase::Unzip,
        Phase::Translate,
        Phase::Fold,
        Phase::Rest,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Phase::Assemble => "1. Building a strand",
            Phase::Pair => "2. Copying: base pairing",
            Phase::Twist => "2. Copying: the double helix",
            Phase::Unzip => "2. Copying: unzipping",
            Phase::Translate => "3. Translation",
            Phase::Fold => "3. Folding",
            Phase::Rest => "A protein is born",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Phase::Assemble => "Free nucleotides drift in and bond to the strand, one at a time.",
            Phase::Pair => {
                "Each base attracts its partner. Hydrogen bonds hold the pairs: two for A with U (or T), three for C with G."
            }
            Phase::Twist => "The paired strands twist into a double helix.",
            Phase::Unzip => {
                "The helix unzips: now there are two strands, the original and its complement."
            }
            Phase::Translate => {
                "A ribosome slides along the strand to the start codon AUG, then reads three bases at a time. Each codon names an amino acid."
            }
            Phase::Fold => {
                "At the stop codon the chain is released and folds into a compact coil. Its shape decides what it can do."
            }
            Phase::Rest => "That protein can help the strand that made it copy itself faster.",
        }
    }
}

/// Where everything is at one moment. Counts are fractional (see the module notes).
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub phase: Phase,
    /// How far through the phase, 0 to 1.
    pub progress: f32,
    /// Beads that have joined the strand (the next one is `fract` of the way).
    pub joined: f32,
    /// Complementary bases docked.
    pub docked: f32,
    /// 0 flat, 1 a full helix.
    pub twist: f32,
    /// Base pairs that have come apart.
    pub unzipped: f32,
    /// Where the ribosome sits, as a base index (its first base).
    pub ribosome_at: f32,
    /// Codons read so far (fractional: the last is being decoded).
    pub codons_read: f32,
    /// Amino acids fully added to the chain.
    pub amino_acids: usize,
    /// The codon being decoded and what it means, while the ribosome is reading.
    pub codon: Option<(String, Codon)>,
    /// The chain has left the ribosome.
    pub released: bool,
    /// 0 stretched out, 1 folded.
    pub fold: f32,
}

/// The scene: a gene, the spelling, and a clock.
#[derive(Clone, Debug)]
pub struct Formation {
    gene: String,
    copy: String,
    frame: ReadingFrame,
    protein: String,
    pub dna: bool,
    pub paused: bool,
    speed: f32,
    time: f32,
}

impl Default for Formation {
    fn default() -> Self {
        Self::with_gene(DEFAULT_GENE).expect("the default gene is a gene")
    }
}

impl Formation {
    /// A scene for `gene` (letters A, C, G, U), or `None` if it has no gene in it
    /// (no AUG, or a protein shorter than two amino acids).
    pub fn with_gene(gene: &str) -> Option<Self> {
        let frame = reading_frame(gene)?;
        let protein = translate(gene)?;
        Some(Self {
            gene: gene.to_string(),
            copy: complement_strand(gene),
            frame,
            protein,
            dna: false,
            paused: false,
            speed: 1.0,
            time: 0.0,
        })
    }

    pub fn gene(&self) -> &str {
        &self.gene
    }

    /// The strand built by base pairing: the complement of the gene.
    pub fn copy(&self) -> &str {
        &self.copy
    }

    pub fn frame(&self) -> &ReadingFrame {
        &self.frame
    }

    pub fn protein(&self) -> &str {
        &self.protein
    }

    /// The protein's fitness, as the soup computes it.
    pub fn fitness(&self) -> f64 {
        protein_fitness(&self.protein)
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(MIN_SPEED, MAX_SPEED);
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    /// Starts over from the first phase.
    pub fn restart(&mut self) {
        self.time = 0.0;
    }

    /// Swaps in another gene and starts over; keeps the spelling, speed and pause.
    /// Returns false (and changes nothing) if `gene` has no gene in it.
    pub fn set_gene(&mut self, gene: &str) -> bool {
        match Self::with_gene(gene) {
            Some(next) => {
                *self = Self {
                    dna: self.dna,
                    paused: self.paused,
                    speed: self.speed,
                    ..next
                };
                true
            }
            None => false,
        }
    }

    /// Seconds each phase lasts for this gene, in order.
    pub fn durations(&self) -> [(Phase, f32); 7] {
        let n = self.gene.len() as f32;
        let read =
            self.frame.start as f32 * SCAN_TIME + self.frame.codons.len() as f32 * CODON_TIME;
        [
            (Phase::Assemble, n * JOIN_TIME),
            (Phase::Pair, n * DOCK_TIME),
            (Phase::Twist, TWIST_TIME),
            (Phase::Unzip, UNZIP_TIME_BASE + n * UNZIP_TIME_PER_BASE),
            (Phase::Translate, read),
            (Phase::Fold, FOLD_TIME),
            (Phase::Rest, REST_TIME),
        ]
    }

    /// Length of the whole loop, in seconds.
    pub fn total_time(&self) -> f32 {
        self.durations().iter().map(|(_, d)| d).sum()
    }

    /// Moves the clock on by `dt` real seconds (scaled by the speed), looping.
    /// Does nothing while paused.
    pub fn advance(&mut self, dt: f32) {
        if !self.paused {
            self.time = (self.time + dt.max(0.0) * self.speed).rem_euclid(self.total_time());
        }
    }

    /// Jumps to the start of `phase`.
    pub fn seek(&mut self, phase: Phase) {
        let mut t = 0.0;
        for (p, d) in self.durations() {
            if p == phase {
                break;
            }
            t += d;
        }
        // a hair past the boundary, so rounding cannot leave us in the previous phase
        self.time = t + 1e-3;
    }

    /// The letter to draw for `base` during `phase`: T for U in DNA mode, except
    /// while reading, when the ribosome sees messenger RNA.
    pub fn shown(&self, phase: Phase, base: char) -> char {
        if self.dna && base == 'U' && phase < Phase::Translate {
            'T'
        } else {
            base
        }
    }

    /// The partner of `base` in the chosen spelling, as drawn: A pairs with U or T.
    pub fn pair_of(&self, base: char) -> char {
        self.shown(Phase::Assemble, crate::model::complement(base))
    }

    /// Hydrogen bonds between a base and its partner: 2 for A, 3 for C and G.
    pub fn bonds_between(base: char) -> usize {
        if matches!(base, 'C' | 'G') { 3 } else { 2 }
    }

    /// The phase and the progress through it at the current time.
    pub fn locate(&self) -> (Phase, f32) {
        self.locate_at(self.time)
    }

    fn locate_at(&self, time: f32) -> (Phase, f32) {
        let mut t = time;
        for (phase, d) in self.durations() {
            if t < d {
                return (phase, (t / d).clamp(0.0, 1.0));
            }
            t -= d;
        }
        (Phase::Rest, 1.0)
    }

    /// Where everything is now.
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot_at(self.time)
    }

    pub fn snapshot_at(&self, time: f32) -> Snapshot {
        let (phase, progress) = self.locate_at(time);
        let n = self.gene.len() as f32;
        let before = |p: Phase| phase > p;
        let joined = if before(Phase::Assemble) {
            n
        } else if phase == Phase::Assemble {
            n * progress
        } else {
            0.0
        };
        let docked = if before(Phase::Pair) {
            n
        } else if phase == Phase::Pair {
            n * progress
        } else {
            0.0
        };
        let twist = match phase {
            Phase::Twist => progress,
            Phase::Unzip => 1.0 - progress * 0.8,
            _ => 0.0,
        };
        let unzipped = if before(Phase::Unzip) {
            n
        } else if phase == Phase::Unzip {
            n * progress
        } else {
            0.0
        };

        let reads = self.frame.codons.len() as f32;
        let (ribosome_at, codons_read) = if phase == Phase::Translate {
            let seconds = progress * self.durations()[4].1;
            let scan = self.frame.start as f32 * SCAN_TIME;
            if seconds < scan {
                (seconds / SCAN_TIME, 0.0)
            } else {
                let k = ((seconds - scan) / CODON_TIME).min(reads);
                (self.frame.start as f32 + 3.0 * k, k)
            }
        } else if before(Phase::Translate) {
            (self.frame.start as f32 + 3.0 * reads, reads)
        } else {
            (0.0, 0.0)
        };
        let whole = (codons_read.floor() as usize).min(self.frame.codons.len());
        let amino_acids = self.frame.codons[..whole]
            .iter()
            .filter(|(_, c)| matches!(c, Codon::Amino(_)))
            .count();
        let reading = phase == Phase::Translate
            && codons_read < reads
            && (codons_read > 0.0 || ribosome_at >= self.frame.start as f32);
        let codon = reading.then(|| self.frame.codons[whole].clone());
        let released =
            before(Phase::Translate) || (phase == Phase::Translate && codons_read >= reads);
        let fold = match phase {
            Phase::Fold => progress,
            Phase::Rest => 1.0,
            _ => 0.0,
        };
        Snapshot {
            phase,
            progress,
            joined,
            docked,
            twist,
            unzipped,
            ribosome_at,
            codons_read,
            amino_acids,
            codon,
            released,
            fold,
        }
    }
}

/// The name of an amino acid from its one-letter code.
pub fn amino_name(letter: char) -> &'static str {
    match letter {
        'A' => "Alanine",
        'R' => "Arginine",
        'N' => "Asparagine",
        'D' => "Aspartate",
        'C' => "Cysteine",
        'E' => "Glutamate",
        'Q' => "Glutamine",
        'G' => "Glycine",
        'H' => "Histidine",
        'I' => "Isoleucine",
        'L' => "Leucine",
        'K' => "Lysine",
        'M' => "Methionine",
        'F' => "Phenylalanine",
        'P' => "Proline",
        'S' => "Serine",
        'T' => "Threonine",
        'W' => "Tryptophan",
        'Y' => "Tyrosine",
        'V' => "Valine",
        _ => "?",
    }
}

/// A random gene of at most [`MAX_PROTEIN_GENE_BASES`] bases: a short lead, the
/// start codon, 2 to 5 sense codons, a stop codon and a short tail. `translate`
/// always finds a protein in it.
pub fn random_gene(rng: &mut impl Rng) -> String {
    const BASES: [char; 4] = ['A', 'C', 'G', 'U'];
    loop {
        let mut gene: String = (0..rng.random_range(0..=3))
            .map(|_| BASES[rng.random_range(0..4)])
            .collect();
        let lead = gene.len();
        gene.push_str("AUG");
        for _ in 0..rng.random_range(2..=5) {
            loop {
                let codon: String = (0..3).map(|_| BASES[rng.random_range(0..4)]).collect();
                if matches!(crate::model::read_codon(&codon), Some(Codon::Amino(_))) {
                    gene.push_str(&codon);
                    break;
                }
            }
        }
        gene.push_str(["UAA", "UAG", "UGA"][rng.random_range(0..3)]);
        let room = MAX_PROTEIN_GENE_BASES.saturating_sub(gene.len()).min(3);
        gene.extend((0..rng.random_range(0..=room)).map(|_| BASES[rng.random_range(0..4)]));
        // A random lead could contain AUG by chance; keep only genes that start where we meant.
        if gene.len() <= MAX_PROTEIN_GENE_BASES
            && reading_frame(&gene).is_some_and(|f| f.start == lead)
            && translate(&gene).is_some()
        {
            return gene;
        }
    }
}

#[cfg(test)]
mod tests;
