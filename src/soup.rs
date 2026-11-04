//! The primordial soup: a 2D environment where amino acid chains are generated,
//! drift with temperature-dependent Brownian motion, mutate and recombine, and
//! where we watch for a target pattern ("METHINKS") to emerge.

use crate::events::{ChainEvent, EventLog};
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
    /// How every chain in the soup came to be. Chains added with `add_molecule`
    /// or by writing `molecules` directly may have no entry, so readers must
    /// treat a missing creation event as normal.
    pub events: EventLog,
    /// Steps simulated so far. Seed chains are logged at step 0; events during
    /// the n-th `simulate_step` are logged at step n.
    pub step: u64,
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
            events: EventLog::new(),
            step: 0,
            width,
            height,
            total_reactions: 0,
            reactions_this_step: 0,
            rng,
        };
        for _ in 0..INITIAL_AMINO_ACIDS {
            soup.spawn_chain();
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

    /// Adds a new random chain and logs it as generated. The one place chains
    /// appear from nothing, so seeding and later generation cannot drift apart.
    fn spawn_chain(&mut self) {
        let mut chain = Molecule::random_chain(&mut self.rng, 1, 3);
        chain.position = self.random_position();
        chain.id = self.events.new_id();
        self.events.record(
            self.step,
            ChainEvent::Generated { id: chain.id, sequence: chain.sequence().unwrap_or("").to_string() },
        );
        self.molecules.push(chain);
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
            self.spawn_chain();
        }
    }

    /// Advances one step: generation, mutation, recombination, movement. Returns
    /// the sequences that contain the target pattern after the step.
    pub fn simulate_step(&mut self) -> Vec<String> {
        self.step += 1;
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
    /// The child replaces its parent in the soup, but is logged as a new chain
    /// linked to the parent. A mutation that makes no edit (deleting from a
    /// one-residue chain) leaves the chain and the log untouched.
    fn mutate(&mut self) {
        let rng = &mut self.rng;
        for m in &mut self.molecules {
            if m.is_amino_acid() && rng.random::<f64>() < MUTATION_RATE {
                let edit = match rng.random_range(0..3) {
                    0 => m.mutate_substitution(rng),
                    1 => m.mutate_insertion(rng),
                    _ => m.mutate_deletion(rng),
                };
                if let Some((mut child, mutation)) = edit {
                    child.id = self.events.new_id();
                    self.events.record(
                        self.step,
                        ChainEvent::Mutated {
                            id: child.id,
                            parent: m.id,
                            mutation,
                            sequence: child.sequence().unwrap_or("").to_string(),
                        },
                    );
                    *m = child;
                }
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
                // Chains that are empty cannot be crossed (never happens in a
                // normal run); skip them rather than add a duplicate of a parent.
                let Some((mut child, cut)) = a.crossover(b, &mut self.rng) else { continue };
                child.position = Position::new((a.position.x + b.position.x) / 2.0, (a.position.y + b.position.y) / 2.0);
                child.id = self.events.new_id();
                self.events.record(
                    self.step,
                    ChainEvent::Recombined {
                        id: child.id,
                        parents: (a.id, b.id),
                        cut,
                        sequence: child.sequence().unwrap_or("").to_string(),
                    },
                );
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

    use crate::events::Mutation;

    fn busy_soup(seed: u64) -> PrimordialSoup {
        // Hot and long, so all three mutation kinds and recombination occur.
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 3000.0, 7.0, seed);
        for _ in 0..3_000 {
            soup.simulate_step();
        }
        soup
    }

    fn apply(parent: &str, m: &Mutation) -> String {
        let mut chars: Vec<char> = parent.chars().collect();
        match *m {
            Mutation::Substitution { index, from, to } => {
                assert_eq!(chars[index], from);
                chars[index] = to;
            }
            Mutation::Insertion { index, residue } => chars.insert(index, residue),
            Mutation::Deletion { index, residue } => assert_eq!(chars.remove(index), residue),
        }
        chars.into_iter().collect()
    }

    #[test]
    fn every_chain_has_a_creation_event() {
        let soup = busy_soup(21);
        for m in soup.molecules.iter().filter(|m| m.is_amino_acid()) {
            let created = soup.events.creation_of(m.id).expect("chain has no creation event");
            assert_eq!(created.event.sequence(), m.sequence().unwrap());
        }
        // Seed chains are logged at step 0 and the step counter advanced.
        assert_eq!(soup.step, 3_000);
        assert!(soup.events.since(0)[..INITIAL_AMINO_ACIDS].iter().all(|s| s.step == 0));
    }

    #[test]
    fn parents_are_created_before_their_children() {
        let soup = busy_soup(22);
        let all = soup.events.since(0);
        let (mut mutated, mut recombined) = (0, 0);
        for (i, stamped) in all.iter().enumerate() {
            match stamped.event {
                ChainEvent::Mutated { .. } => mutated += 1,
                ChainEvent::Recombined { .. } => recombined += 1,
                ChainEvent::Generated { .. } => {}
            }
            for parent in stamped.event.parents() {
                assert!(
                    all[..i].iter().any(|s| s.event.id() == parent),
                    "parent {parent} of event {i} not created earlier"
                );
            }
        }
        assert!(mutated > 0 && recombined > 0, "run too quiet to test anything");
    }

    #[test]
    fn event_steps_never_go_backwards() {
        let soup = busy_soup(23);
        let steps: Vec<u64> = soup.events.since(0).iter().map(|s| s.step).collect();
        assert!(steps.windows(2).all(|w| w[0] <= w[1]));
        assert!(*steps.last().unwrap() > 0);
    }

    #[test]
    fn recorded_edits_explain_the_children() {
        let soup = busy_soup(24);
        let seq_of = |id| soup.events.creation_of(id).unwrap().event.sequence().to_string();
        let (mut sub, mut ins, mut del) = (0, 0, 0);
        for stamped in soup.events.since(0) {
            match &stamped.event {
                ChainEvent::Mutated { parent, mutation, sequence, .. } => {
                    assert_eq!(&apply(&seq_of(*parent), mutation), sequence);
                    match mutation {
                        Mutation::Substitution { .. } => sub += 1,
                        Mutation::Insertion { .. } => ins += 1,
                        Mutation::Deletion { .. } => del += 1,
                    }
                }
                ChainEvent::Recombined { parents, cut, sequence, .. } => {
                    let expected = format!("{}{}", &seq_of(parents.0)[..cut.0], &seq_of(parents.1)[cut.1..]);
                    assert_eq!(&expected, sequence);
                }
                ChainEvent::Generated { .. } => {}
            }
        }
        assert!(sub > 0 && ins > 0 && del > 0, "saw sub {sub}, ins {ins}, del {del}");
    }

    #[test]
    fn mutated_child_replaces_parent_with_new_id() {
        let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 300.0, 7.0, 25);
        let before: Vec<_> = soup.molecules.iter().map(|m| m.id).collect();
        for _ in 0..2_000 {
            soup.simulate_step();
        }
        let mutated_parents: Vec<_> = soup
            .events
            .since(0)
            .iter()
            .filter_map(|s| match s.event {
                ChainEvent::Mutated { parent, .. } => Some(parent),
                _ => None,
            })
            .collect();
        assert!(!mutated_parents.is_empty());
        // A parent that was mutated is gone from the soup (unless recombination
        // alone kept a copy, which it never does: children are new chains).
        assert!(soup.molecules.iter().all(|m| !mutated_parents.contains(&m.id)));
        assert!(before.iter().any(|id| mutated_parents.contains(id)));
    }

    #[test]
    fn unrecorded_chains_do_not_break_the_log() {
        let mut soup = PrimordialSoup::with_seed(100.0, 100.0, 3000.0, 7.0, 26);
        let mut extra = Molecule::amino_acid("MMMMM".into(), 1.0);
        extra.position = Position::new(5.0, 5.0);
        soup.add_molecule(extra);
        soup.molecules.push(Molecule::amino_acid("KKK".into(), 1.0));
        for _ in 0..2_000 {
            soup.simulate_step();
        }
        let last = soup.events.len().saturating_sub(1);
        let _ = soup.events.ancestors(soup.events.since(last)[0].event.id());
    }

    #[test]
    fn same_seed_gives_same_log() {
        assert_eq!(busy_soup(27).events.since(0), busy_soup(27).events.since(0));
    }

    /// Fingerprint of the final population: sequence, energy and position of every
    /// molecule (FNV-1a), so it also catches any change in RNG draw order.
    fn fingerprint(soup: &PrimordialSoup) -> u64 {
        let mut h: u64 = 1469598103934665603;
        for m in &soup.molecules {
            let t = format!("{:?}|{:.6}|{:.6}|{:.6};", m.sequence(), m.energy, m.position.x, m.position.y);
            for b in t.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(1099511628211);
            }
        }
        h
    }

    /// The expected values were produced by the code at HEAD, before event
    /// logging existed (same seeds, 3000 steps, 300x200 soup).
    #[test]
    fn seeded_runs_match_the_pre_event_log_code() {
        for (seed, temp, len, hash) in
            [(11, 300.0, 50, 0xdfe75a6faf00ccc8), (12, 3000.0, 50, 0xa74eda001e6d664f)]
        {
            let mut soup = PrimordialSoup::with_seed(300.0, 200.0, temp, 7.0, seed);
            for _ in 0..3_000 {
                soup.simulate_step();
            }
            assert_eq!(soup.molecules.len(), len);
            assert_eq!(fingerprint(&soup), hash, "seed {seed} diverged from the old behaviour");
        }
    }
}
