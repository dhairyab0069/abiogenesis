//! The primordial soup: a 2D environment where amino acid chains are generated,
//! drift with temperature-dependent Brownian motion, mutate and recombine, and
//! where we watch for a target pattern ("METHINKS") to emerge.

use crate::model::{Molecule, Position};
use rand::{Rng, SeedableRng, rngs::StdRng};

const MOVEMENT_SPEED: f64 = 2.0;
const INITIAL_AMINO_ACIDS: usize = 20;
/// Probability per step that a new chain is generated.
const AMINO_ACID_GENERATION_RATE: f64 = 0.005;
/// Population cap for chains (generation) and for the whole soup (recombination).
pub const MAX_AMINO_ACIDS: usize = 50;
/// The sequence we watch for.
pub const TARGET_PATTERN: &str = "METHINKS";
/// Probability per step that a chain mutates.
const MUTATION_RATE: f64 = 0.01;
/// Probability that a picked pair of chains recombines.
const COMBINATION_RATE: f64 = 0.01;
const MAX_COMBINATIONS_PER_STEP: usize = 10;
/// Bounds the pair-picking loop, which could otherwise spin forever when the
/// soup has fewer than two chains.
const MAX_PAIR_ATTEMPTS: usize = 2_000;

pub struct PrimordialSoup {
    pub molecules: Vec<Molecule>,
    pub temperature: f64,
    pub ph: f64,
    width: f64,
    height: f64,
    total_reactions: u64,
    reactions_this_step: u64,
    rng: StdRng,
}

impl PrimordialSoup {
    pub fn new(width: f64, height: f64, temperature: f64, ph: f64) -> Self {
        Self::with_rng(width, height, temperature, ph, StdRng::from_os_rng())
    }

    pub fn with_seed(width: f64, height: f64, temperature: f64, ph: f64, seed: u64) -> Self {
        Self::with_rng(width, height, temperature, ph, StdRng::seed_from_u64(seed))
    }

    fn with_rng(width: f64, height: f64, temperature: f64, ph: f64, rng: StdRng) -> Self {
        let mut soup = Self {
            molecules: Vec::new(),
            temperature,
            ph,
            width,
            height,
            total_reactions: 0,
            reactions_this_step: 0,
            rng,
        };
        for _ in 0..INITIAL_AMINO_ACIDS {
            let chain = soup.random_chain_somewhere();
            soup.molecules.push(chain);
        }
        soup
    }

    /// The molecule simulation from the launcher: 20 seed chains plus 50 water and
    /// 50 methane molecules at 300 K, pH 7.
    pub fn standard(width: f64, height: f64) -> Self {
        let mut soup = Self::new(width, height, 300.0, 7.0);
        soup.add_small_molecules();
        soup
    }

    pub fn add_small_molecules(&mut self) {
        for _ in 0..50 {
            for mut m in [Molecule::water(), Molecule::methane()] {
                m.position = self.random_position();
                self.add_molecule(m);
            }
        }
    }

    fn random_position(&mut self) -> Position {
        Position::new(self.rng.random::<f64>() * self.width, self.rng.random::<f64>() * self.height)
    }

    fn random_chain_somewhere(&mut self) -> Molecule {
        let mut chain = Molecule::random_chain(&mut self.rng, 1, 3);
        chain.position = self.random_position();
        chain
    }

    /// Adds a molecule if its position is inside the soup.
    pub fn add_molecule(&mut self, molecule: Molecule) {
        if self.is_valid_position(&molecule.position) {
            self.molecules.push(molecule);
        }
    }

    fn is_valid_position(&self, p: &Position) -> bool {
        p.x >= 0.0 && p.x < self.width && p.y >= 0.0 && p.y < self.height
    }

    pub fn amino_acid_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_amino_acid()).count()
    }

    fn generate_amino_acids(&mut self) {
        if self.amino_acid_count() < MAX_AMINO_ACIDS
            && self.rng.random::<f64>() < AMINO_ACID_GENERATION_RATE
        {
            let chain = self.random_chain_somewhere();
            self.molecules.push(chain);
        }
    }

    /// Advances one step: generation, mutation, recombination, movement. Returns
    /// the sequences that contain the target pattern after the step.
    pub fn simulate_step(&mut self) -> Vec<String> {
        self.reactions_this_step = 0;
        self.generate_amino_acids();
        self.mutate();
        self.recombine();
        let scale = MOVEMENT_SPEED * (self.temperature / 300.0);
        let (w, h) = (self.width, self.height);
        for m in &mut self.molecules {
            let dx = (self.rng.random::<f64>() - 0.5) * scale;
            let dy = (self.rng.random::<f64>() - 0.5) * scale;
            m.position.x = (m.position.x + dx).clamp(0.0, w);
            m.position.y = (m.position.y + dy).clamp(0.0, h);
        }
        self.molecules
            .iter()
            .filter_map(|m| m.sequence())
            .filter(|s| s.contains(TARGET_PATTERN))
            .map(str::to_string)
            .collect()
    }

    /// Each chain has a small chance to mutate by substitution, insertion or deletion.
    fn mutate(&mut self) {
        let rng = &mut self.rng;
        for m in &mut self.molecules {
            if m.is_amino_acid() && rng.random::<f64>() < MUTATION_RATE {
                *m = match rng.random_range(0..3) {
                    0 => m.mutate_substitution(rng),
                    1 => m.mutate_insertion(rng),
                    _ => m.mutate_deletion(rng),
                };
            }
        }
    }

    /// Picks random pairs from the current population and occasionally adds their
    /// crossover child at the pair's midpoint, while the soup is under the cap.
    fn recombine(&mut self) {
        let snapshot_len = self.molecules.len();
        if snapshot_len < 2 {
            return;
        }
        let mut combinations = 0;
        for _ in 0..MAX_PAIR_ATTEMPTS {
            if combinations >= MAX_COMBINATIONS_PER_STEP || self.molecules.len() >= MAX_AMINO_ACIDS {
                break;
            }
            let i = self.rng.random_range(0..snapshot_len);
            let j = self.rng.random_range(0..snapshot_len);
            if i == j {
                continue;
            }
            let (a, b) = (&self.molecules[i], &self.molecules[j]);
            if a.is_amino_acid() && b.is_amino_acid() && self.rng.random::<f64>() < COMBINATION_RATE {
                let mut child = a.crossover(b, &mut self.rng);
                child.position = Position::new((a.position.x + b.position.x) / 2.0, (a.position.y + b.position.y) / 2.0);
                self.molecules.push(child);
                combinations += 1;
            }
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }

    /// Resizes the soup, rescaling every molecule's position to match.
    pub fn resize(&mut self, width: f64, height: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let (sx, sy) = (width / self.width, height / self.height);
        for m in &mut self.molecules {
            m.position.x *= sx;
            m.position.y *= sy;
        }
        self.width = width;
        self.height = height;
    }

    /// Reactions are not modelled yet; always 0.
    pub fn total_reactions(&self) -> u64 {
        self.total_reactions
    }

    /// Reactions are not modelled yet; always 0.
    pub fn reactions_this_step(&self) -> u64 {
        self.reactions_this_step
    }

    /// Molecule counts by name, sorted by name.
    pub fn counts(&self) -> Vec<(String, usize)> {
        let mut counts = std::collections::BTreeMap::new();
        for m in &self.molecules {
            *counts.entry(m.name()).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_seed_chains_inside_bounds() {
        let soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 1);
        assert_eq!(soup.molecules.len(), INITIAL_AMINO_ACIDS);
        assert!(soup.molecules.iter().all(|m| m.is_amino_acid() && soup.is_valid_position(&m.position)));
    }

    #[test]
    fn standard_soup_adds_water_and_methane() {
        let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 2);
        soup.add_small_molecules();
        let counts = soup.counts();
        let get = |n: &str| counts.iter().find(|(k, _)| k == n).map(|(_, c)| *c);
        assert_eq!(get("H2O"), Some(50));
        assert_eq!(get("CH4"), Some(50));
        assert_eq!(soup.amino_acid_count(), INITIAL_AMINO_ACIDS);
    }

    #[test]
    fn molecules_stay_in_bounds_and_chains_stay_capped() {
        let mut soup = PrimordialSoup::with_seed(200.0, 100.0, 3000.0, 7.0, 3);
        for _ in 0..5_000 {
            soup.simulate_step();
        }
        assert!(soup.molecules.iter().all(|m| (0.0..=200.0).contains(&m.position.x)
            && (0.0..=100.0).contains(&m.position.y)));
        // Generation stops at the cap; recombination can add at most 10 past it.
        assert!(soup.amino_acid_count() <= MAX_AMINO_ACIDS + MAX_COMBINATIONS_PER_STEP);
    }

    #[test]
    fn recombination_terminates_with_too_few_chains() {
        let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 4);
        soup.molecules.truncate(1);
        soup.molecules.push(Molecule::water());
        soup.simulate_step();
    }

    #[test]
    fn resize_rescales_positions() {
        let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 5);
        soup.molecules[0].position = Position::new(50.0, 25.0);
        soup.resize(200.0, 50.0);
        assert_eq!(soup.molecules[0].position, Position::new(100.0, 12.5));
    }

    #[test]
    fn reports_target_pattern() {
        let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 6);
        let mut m = Molecule::amino_acid("XXMETHINKSXX".into(), 1.0);
        m.position = Position::new(10.0, 10.0);
        soup.molecules = vec![m];
        let found = soup.simulate_step();
        assert_eq!(found, vec!["XXMETHINKSXX".to_string()]);
    }
}
