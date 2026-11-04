//! Stage 3 chemistry: free nucleotides join into RNA-like strands, strands copy
//! themselves, strands and lipids slowly fall apart, and lipids cling together.
//!
//! Design choices:
//! - **Everything is a small rule with a rate.** Each step a few things may
//!   happen, each with its own probability, so a visitor can read the rule and
//!   predict the behaviour.
//! - **Drying joins, wetness breaks.** Nucleotides link up faster in a dry pool
//!   (water has to leave for the bond to form), and strands fall apart faster
//!   when it is wet. That ties stage 3 to the pool of stage 2.
//! - **Copying needs raw material.** A copy uses up as many free nucleotides
//!   (within reach) as the template is long. When they run out, replication
//!   stops, so strands compete for building blocks. That competition is what
//!   makes fitness matter.
//! - **Protocells help.** A strand inside a droplet falls apart half as fast and
//!   copies itself twice as fast, because everything it needs stays close.
//! - **Off unless asked for.** `add_prebiotic_chemistry` switches it on. A plain
//!   soup never draws a random number here, so seeded runs of stages 1 and 2
//!   stay exactly the same.

use super::PrimordialSoup;
use crate::model::{BASES, MOTIF, Molecule, Position, complement_strand};
use rand::Rng;

const START_NUCLEOTIDES: usize = 30;
const START_LIPIDS: usize = 12;
pub(super) const MAX_FREE_NUCLEOTIDES: usize = 80;
pub(super) const MAX_LIPIDS: usize = 24;
/// Strands the soup holds at most, and the longest one.
pub const MAX_STRANDS: usize = 40;
pub const MAX_STRAND_LENGTH: usize = 24;
/// Chance per step that the soup makes a new nucleotide, or a new lipid.
const NUCLEOTIDE_SUPPLY: f64 = 0.12;
const LIPID_SUPPLY: f64 = 0.01;
/// Two free nucleotides closer than this can bond. A strand reaches further, to
/// pick up a nucleotide that drifts by.
const JOIN_DISTANCE: f64 = 14.0;
const EXTEND_DISTANCE: f64 = 40.0;
const POLYMERIZATION_RATE: f64 = 0.04;
/// Random nucleotides tried per step.
const JOIN_ATTEMPTS: usize = 12;
/// A strand finds this many free nucleotides within reach, or it cannot copy.
const TEMPLATE_RADIUS: f64 = 100.0;
const REPLICATION_RATE: f64 = 0.01;
const CATALYST_BOOST: f64 = 4.0;
const SHELTER_REPLICATION_BOOST: f64 = 2.0;
/// Chance that a base is copied wrongly.
pub(super) const COPY_ERROR: f64 = 0.03;
const STRAND_DECAY: f64 = 0.0006;
const LIPID_DECAY: f64 = 0.0005;
const SHELTER_DECAY: f64 = 0.5;
/// How strongly a lipid is pulled toward the nearest lipid, and from how far.
const LIPID_PULL: f64 = 0.03;
const LIPID_REACH: f64 = 90.0;

/// Chance per step that a template that can copy itself does. Catalysts are
/// faster, so are strands whose protein is fit (the gene boost), and so is
/// anything inside a protocell.
pub(super) fn replication_rate(sequence: &str, sheltered: bool) -> f64 {
    let mut rate = REPLICATION_RATE * super::protein::gene_boost(sequence);
    if sequence.contains(MOTIF) {
        rate *= CATALYST_BOOST;
    }
    if sheltered {
        rate *= SHELTER_REPLICATION_BOOST;
    }
    rate
}

/// Chance per step that a molecule falls apart. Fit molecules last longer, long
/// strands are more fragile, water speeds it up and a protocell slows it down.
/// Nothing else decays.
pub(super) fn decay_probability(m: &Molecule, wetness: f64, sheltered: bool) -> f64 {
    let shelter = if sheltered { SHELTER_DECAY } else { 1.0 };
    if let Some(seq) = m.strand_sequence() {
        STRAND_DECAY * (1.0 - m.fitness()) * (0.5 + wetness) * (1.0 + seq.len() as f64 * 0.03) * shelter
    } else if m.is_lipid() {
        LIPID_DECAY * (0.5 + wetness) * shelter
    } else {
        0.0
    }
}

/// The strand a template builds: its complement, with each base wrong with
/// probability `error`.
pub(super) fn copy_strand(rng: &mut impl Rng, template: &str, error: f64) -> String {
    complement_strand(template)
        .chars()
        .map(|c| if rng.random::<f64>() < error { BASES[rng.random_range(0..BASES.len())] } else { c })
        .collect()
}

impl PrimordialSoup {
    /// Switches the stage 3 chemistry on and adds some free nucleotides and lipids.
    pub fn add_prebiotic_chemistry(&mut self) {
        self.chemistry = true;
        for _ in 0..START_NUCLEOTIDES {
            self.spawn_nucleotide();
        }
        for _ in 0..START_LIPIDS {
            self.spawn_lipid();
        }
    }

    fn spawn_nucleotide(&mut self) {
        let mut n = Molecule::nucleotide(BASES[self.rng.random_range(0..BASES.len())]);
        n.position = self.random_position();
        self.molecules.push(n);
    }

    fn spawn_lipid(&mut self) {
        let mut l = Molecule::lipid();
        l.position = self.random_position();
        self.molecules.push(l);
    }

    pub fn nucleotide_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_nucleotide()).count()
    }

    pub fn strand_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_strand()).count()
    }

    pub fn lipid_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_lipid()).count()
    }

    /// Strands that carry the catalytic motif.
    pub fn catalyst_count(&self) -> usize {
        self.molecules.iter().filter(|m| m.is_catalyst()).count()
    }

    pub fn longest_strand(&self) -> usize {
        self.molecules.iter().filter_map(|m| m.strand_sequence()).map(str::len).max().unwrap_or(0)
    }

    /// The average fitness of the strands, or 0 when there are none.
    pub fn mean_strand_fitness(&self) -> f64 {
        let strands: Vec<f64> = self.molecules.iter().filter(|m| m.is_strand()).map(Molecule::fitness).collect();
        if strands.is_empty() { 0.0 } else { strands.iter().sum::<f64>() / strands.len() as f64 }
    }

    /// New strands that started from two free nucleotides.
    pub fn strands_formed(&self) -> u64 {
        self.strands_formed
    }

    /// Strands built by copying another strand.
    pub fn strand_copies(&self) -> u64 {
        self.strand_copies
    }

    pub fn strands_decayed(&self) -> u64 {
        self.strands_decayed
    }

    pub(super) fn sheltered(&self, p: &Position) -> bool {
        self.protocells.iter().any(|c| c.contains(p.x, p.y))
    }

    /// One step of chemistry. Does nothing (and draws no random numbers) unless
    /// the chemistry was switched on.
    pub(super) fn step_chemistry(&mut self) {
        if !self.chemistry {
            return;
        }
        if self.nucleotide_count() < MAX_FREE_NUCLEOTIDES && self.rng.random::<f64>() < NUCLEOTIDE_SUPPLY {
            self.spawn_nucleotide();
        }
        if self.lipid_count() < MAX_LIPIDS && self.rng.random::<f64>() < LIPID_SUPPLY {
            self.spawn_lipid();
        }
        self.polymerize();
        self.replicate();
        self.express_genes();
        self.decay();
        self.gather_lipids();
    }

    /// Free nucleotides bond to a neighbour: two of them start a strand, and one
    /// next to a strand extends it. Drying makes bonding up to 4 times as likely.
    pub(super) fn polymerize(&mut self) {
        let dry = 1.0 + 3.0 * (1.0 - self.wetness);
        for _ in 0..JOIN_ATTEMPTS {
            let free: Vec<usize> = (0..self.molecules.len()).filter(|&i| self.molecules[i].is_nucleotide()).collect();
            if free.is_empty() {
                return;
            }
            let i = free[self.rng.random_range(0..free.len())];
            if self.rng.random::<f64>() >= POLYMERIZATION_RATE * dry {
                continue;
            }
            let (here, base) = (self.molecules[i].position, self.molecules[i].base().unwrap_or('A'));
            let partner = (0..self.molecules.len())
                .filter(|&j| j != i)
                .filter(|&j| {
                    let m = &self.molecules[j];
                    m.is_nucleotide() || m.strand_sequence().is_some_and(|s| s.len() < MAX_STRAND_LENGTH)
                })
                .map(|j| (j, self.molecules[j].position.distance_to(&here)))
                .filter(|&(j, d)| d <= if self.molecules[j].is_strand() { EXTEND_DISTANCE } else { JOIN_DISTANCE })
                .min_by(|a, b| a.1.total_cmp(&b.1));
            let Some((j, _)) = partner else { continue };
            if let Some(seq) = self.molecules[j].strand_sequence() {
                let mut longer = Molecule::strand(format!("{seq}{base}"));
                longer.position = self.molecules[j].position;
                self.molecules[j] = longer;
                self.molecules.swap_remove(i);
            } else if self.strand_count() < MAX_STRANDS {
                let other = self.molecules[j].base().unwrap_or('A');
                let seq = if self.rng.random::<bool>() { format!("{base}{other}") } else { format!("{other}{base}") };
                let there = self.molecules[j].position;
                let mut strand = Molecule::strand(seq);
                strand.position = Position::new((here.x + there.x) / 2.0, (here.y + there.y) / 2.0);
                self.molecules.swap_remove(i.max(j));
                self.molecules.swap_remove(i.min(j));
                self.molecules.push(strand);
                self.strands_formed += 1;
            }
        }
    }

    /// Strands copy themselves, if enough free nucleotides lie within reach to
    /// build the copy. The copy is the complement of the template, with the odd
    /// wrong base.
    pub(super) fn replicate(&mut self) {
        let templates: Vec<(Position, String)> = self
            .molecules
            .iter()
            .filter_map(|m| m.strand_sequence().map(|s| (m.position, s.to_string())))
            .collect();
        for (at, sequence) in templates {
            if self.strand_count() >= MAX_STRANDS {
                return;
            }
            if self.rng.random::<f64>() >= replication_rate(&sequence, self.sheltered(&at)) {
                continue;
            }
            let mut near: Vec<(usize, f64)> = (0..self.molecules.len())
                .filter(|&i| self.molecules[i].is_nucleotide())
                .map(|i| (i, self.molecules[i].position.distance_to(&at)))
                .filter(|&(_, d)| d <= TEMPLATE_RADIUS)
                .collect();
            if near.len() < sequence.len() {
                continue;
            }
            near.sort_by(|a, b| a.1.total_cmp(&b.1));
            let mut used: Vec<usize> = near[..sequence.len()].iter().map(|&(i, _)| i).collect();
            used.sort_unstable_by(|a, b| b.cmp(a));
            for i in used {
                self.molecules.swap_remove(i);
            }
            let mut copy = Molecule::strand(copy_strand(&mut self.rng, &sequence, COPY_ERROR));
            copy.position = Position::new(
                (at.x + self.rng.random_range(-12.0..12.0)).clamp(0.0, self.width),
                (at.y + self.rng.random_range(-12.0..12.0)).clamp(0.0, self.height),
            );
            self.molecules.push(copy);
            self.strand_copies += 1;
        }
    }

    /// Strands and lipids fall apart now and then (see `decay_probability`).
    fn decay(&mut self) {
        let wetness = self.wetness;
        let strands_before = self.strand_count();
        let Self { molecules, protocells, rng, .. } = self;
        molecules.retain(|m| {
            let sheltered = protocells.iter().any(|c| c.contains(m.position.x, m.position.y));
            let p = decay_probability(m, wetness, sheltered);
            p <= 0.0 || rng.random::<f64>() >= p
        });
        self.strands_decayed += (strands_before - self.strand_count()) as u64;
    }

    /// Each lipid drifts a little toward the nearest other lipid, so they clump.
    fn gather_lipids(&mut self) {
        let lipids: Vec<(usize, Position)> =
            self.molecules.iter().enumerate().filter(|(_, m)| m.is_lipid()).map(|(i, m)| (i, m.position)).collect();
        if lipids.len() < 2 {
            return;
        }
        for &(i, here) in &lipids {
            let nearest = lipids
                .iter()
                .filter(|&&(j, _)| j != i)
                .map(|&(_, p)| (p, p.distance_to(&here)))
                .filter(|&(_, d)| d <= LIPID_REACH && d > 6.0)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((target, _)) = nearest {
                let m = &mut self.molecules[i];
                m.position.x += (target.x - here.x) * LIPID_PULL;
                m.position.y += (target.y - here.y) * LIPID_PULL;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    fn at(mut m: Molecule, x: f64, y: f64) -> Molecule {
        m.position = Position::new(x, y);
        m
    }

    /// A dry, chemistry-enabled soup holding exactly `molecules`.
    fn soup_with(molecules: Vec<Molecule>) -> PrimordialSoup {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 31);
        soup.molecules = molecules;
        soup.chemistry = true;
        soup.set_environment(Environment { base_wetness: 0.0, ..Environment::default() });
        soup
    }

    #[test]
    fn a_plain_soup_has_no_stage_3_chemistry() {
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 300.0, 7.0, 32);
        for _ in 0..500 {
            soup.simulate_step();
        }
        assert_eq!(soup.nucleotide_count() + soup.strand_count() + soup.lipid_count(), 0);
    }

    #[test]
    fn the_standard_soup_starts_with_nucleotides_and_lipids() {
        let soup = PrimordialSoup::standard(800.0, 600.0);
        assert_eq!(soup.nucleotide_count(), START_NUCLEOTIDES);
        assert_eq!(soup.lipid_count(), START_LIPIDS);
        assert_eq!(soup.strand_count(), 0);
    }

    #[test]
    fn neighbouring_nucleotides_join_into_a_strand() {
        let mut soup = soup_with(vec![at(Molecule::nucleotide('G'), 100.0, 290.0), at(Molecule::nucleotide('U'), 104.0, 290.0)]);
        for _ in 0..200 {
            soup.polymerize();
        }
        assert_eq!(soup.strands_formed(), 1);
        assert_eq!(soup.strand_count(), 1);
        let seq = soup.molecules.iter().find_map(|m| m.strand_sequence()).unwrap();
        assert!(seq == "GU" || seq == "UG");
    }

    #[test]
    fn a_nucleotide_next_to_a_strand_extends_it() {
        let mut soup = soup_with(vec![at(Molecule::strand("GG".into()), 100.0, 290.0), at(Molecule::nucleotide('U'), 103.0, 290.0)]);
        for _ in 0..200 {
            soup.polymerize();
        }
        assert_eq!(soup.molecules.len(), 1);
        assert_eq!(soup.molecules[0].strand_sequence(), Some("GGU"));
        assert_eq!(soup.strands_formed(), 0, "extending is not a new strand");
    }

    #[test]
    fn distant_nucleotides_do_not_join() {
        let mut soup = soup_with(vec![at(Molecule::nucleotide('A'), 20.0, 290.0), at(Molecule::nucleotide('C'), 300.0, 290.0)]);
        for _ in 0..500 {
            soup.polymerize();
        }
        assert_eq!(soup.strand_count(), 0);
    }

    #[test]
    fn strands_never_pass_the_length_cap() {
        let full = "A".repeat(MAX_STRAND_LENGTH);
        let mut soup = soup_with(vec![at(Molecule::strand(full), 100.0, 290.0), at(Molecule::nucleotide('U'), 103.0, 290.0)]);
        for _ in 0..300 {
            soup.polymerize();
        }
        assert_eq!(soup.longest_strand(), MAX_STRAND_LENGTH);
        assert_eq!(soup.nucleotide_count(), 1);
    }

    #[test]
    fn copy_strand_is_the_complement_without_errors() {
        let mut rng = rand::rng();
        assert_eq!(copy_strand(&mut rng, "GGUACA", 0.0), "CCAUGU");
        let wrong = (0..50).filter(|_| copy_strand(&mut rng, "AAAAAAAAAA", 1.0) != "UUUUUUUUUU").count();
        assert!(wrong > 40, "errors at rate 1 should almost always change the copy");
    }

    #[test]
    fn a_strand_copies_itself_using_up_nucleotides() {
        let mut molecules = vec![at(Molecule::strand("GGUACA".into()), 100.0, 290.0)];
        molecules.extend((0..10).map(|k| at(Molecule::nucleotide('A'), 90.0 + k as f64 * 3.0, 285.0)));
        let mut soup = soup_with(molecules);
        for _ in 0..3_000 {
            soup.replicate();
            if soup.strand_copies() > 0 {
                break;
            }
        }
        assert_eq!(soup.strand_copies(), 1);
        assert_eq!(soup.strand_count(), 2);
        assert_eq!(soup.nucleotide_count(), 4, "six of the ten nucleotides were used");
        assert!(soup.molecules.iter().filter_map(|m| m.strand_sequence()).all(|s| s.len() == 6));
    }

    #[test]
    fn no_raw_material_means_no_copying() {
        let mut soup = soup_with(vec![at(Molecule::strand("GGUACA".into()), 100.0, 290.0)]);
        for _ in 0..5_000 {
            soup.replicate();
        }
        assert_eq!(soup.strand_copies(), 0);
        let mut far = vec![at(Molecule::strand("GGUACA".into()), 20.0, 290.0)];
        far.extend((0..10).map(|_| at(Molecule::nucleotide('A'), 380.0, 290.0)));
        let mut soup = soup_with(far);
        for _ in 0..5_000 {
            soup.replicate();
        }
        assert_eq!(soup.strand_copies(), 0, "nucleotides out of reach cannot be used");
    }

    #[test]
    fn catalysts_and_sheltered_strands_copy_faster() {
        assert!(replication_rate("AGGUCA", false) > replication_rate("AGAUCA", false));
        assert!(replication_rate("AGAUCA", true) > replication_rate("AGAUCA", false));
        assert!(replication_rate("AGGUCA", true) > replication_rate("AGGUCA", false));
    }

    #[test]
    fn fit_dry_sheltered_strands_last_longest() {
        let fit = Molecule::strand("GGUCAUGC".into());
        let unfit = Molecule::strand("AAAAAAAA".into());
        assert!(decay_probability(&fit, 0.5, false) < decay_probability(&unfit, 0.5, false));
        assert!(decay_probability(&fit, 0.0, false) < decay_probability(&fit, 1.0, false));
        assert!(decay_probability(&fit, 0.5, true) < decay_probability(&fit, 0.5, false));
        let long = Molecule::strand("GGUCAUGCGGUCAUGC".into());
        assert!(decay_probability(&long, 0.5, false) > 0.0);
        assert_eq!(decay_probability(&Molecule::nucleotide('A'), 1.0, false), 0.0);
        assert_eq!(decay_probability(&Molecule::water(), 1.0, false), 0.0);
    }

    #[test]
    fn unfit_strands_vanish_and_are_counted() {
        let mut soup = soup_with((0..30).map(|k| at(Molecule::strand("AAAAAAAA".into()), 10.0 + k as f64 * 12.0, 290.0)).collect());
        soup.set_environment(Environment::default());
        for _ in 0..8_000 {
            soup.decay();
        }
        assert!(soup.strand_count() < 30);
        assert_eq!(soup.strands_decayed() as usize, 30 - soup.strand_count());
    }

    #[test]
    fn lipids_clump_together() {
        let mut soup = soup_with(vec![at(Molecule::lipid(), 100.0, 290.0), at(Molecule::lipid(), 160.0, 290.0)]);
        for _ in 0..200 {
            soup.gather_lipids();
        }
        let gap = soup.molecules[0].position.distance_to(&soup.molecules[1].position);
        assert!(gap < 20.0, "lipids ended {gap} apart");
    }

    #[test]
    fn a_busy_soup_keeps_its_caps_and_stays_in_bounds() {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 33);
        soup.add_prebiotic_chemistry();
        soup.set_environment(Environment::preset(crate::environment::Preset::Both, 300.0, 7.0));
        for _ in 0..6_000 {
            soup.simulate_step();
            assert!(soup.strand_count() <= MAX_STRANDS && soup.longest_strand() <= MAX_STRAND_LENGTH);
            assert!(soup.nucleotide_count() <= MAX_FREE_NUCLEOTIDES + START_NUCLEOTIDES);
        }
        assert!(soup.molecules.iter().all(|m| (0.0..=400.0).contains(&m.position.x) && (0.0..=300.0).contains(&m.position.y)));
        assert!(soup.strands_formed() > 0, "a long dry-and-wet run should make strands");
    }
}
