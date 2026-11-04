//! Protocells: droplets that form where chains crowd as the pool dries.
//!
//! Design choices:
//! - **A grid decides where.** The soup is cut into square grid cells; a cell
//!   with enough chains in it, while the pool is dry, becomes a protocell. That
//!   is a simple stand-in for "stuff gets concentrated, then a boundary forms".
//! - **A protocell is just a circle.** It has no inside state of its own: the
//!   chains inside it are simply the chains whose position is inside the circle.
//!   That keeps it correct when chains mutate (a child replaces its parent) or
//!   new ones are added, with no member list to keep in sync.
//! - **Why they matter.** Chains cannot leave a protocell, they mutate faster
//!   there and they recombine with each other more often. That is the point of
//!   the stage: whoever is inside a droplet gets to experiment more.
//! - **They sit in the pool.** Only chains below the water line can crowd into a
//!   droplet, and a droplet sinks with the water line as the pool dries, carrying
//!   its chains along.
//! - **They burst when the pool refills.** Wet conditions dissolve the boundary
//!   and the chains are free again, which is how a protocell can lose out to a
//!   changing environment.

use super::{MAX_AMINO_ACIDS, PrimordialSoup};
use rand::Rng;
use std::collections::BTreeMap;

/// Side of a grid cell, in pixels.
pub const GRID_CELL: f64 = 60.0;
/// Radius of a protocell, in pixels.
pub const RADIUS: f64 = 28.0;
/// Chains that must share a grid cell before a protocell forms there.
const CROWD: usize = 3;
/// Protocells only form while the pool is drier than this.
const FORM_BELOW_WETNESS: f64 = 0.45;
/// Protocells burst when the pool is wetter than this.
const BURST_ABOVE_WETNESS: f64 = 0.85;
const MAX_PROTOCELLS: usize = 6;
/// How fast (per step) a droplet sinks with the water line.
const SETTLE_SPEED: f64 = 3.0;
/// A chain inside a protocell mutates this many times as often.
pub(super) const MUTATION_BOOST: f64 = 2.0;
/// Chance per step that two chains in the same protocell cross over.
const SHARED_RECOMBINATION_RATE: f64 = 0.02;

/// One droplet.
#[derive(Clone, Debug, PartialEq)]
pub struct Protocell {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    /// The step it formed on.
    pub born_step: u64,
}

impl Protocell {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        (x - self.x).hypot(y - self.y) <= self.radius
    }

    /// The point itself if it is inside, else the nearest point on the boundary
    /// (just inside it, so a chain on the edge still counts as a member).
    pub fn keep_inside(&self, x: f64, y: f64) -> (f64, f64) {
        let d = (x - self.x).hypot(y - self.y);
        if d <= self.radius {
            return (x, y);
        }
        let k = (self.radius * 0.98) / d;
        (self.x + (x - self.x) * k, self.y + (y - self.y) * k)
    }
}

impl PrimordialSoup {
    /// Protocells that have formed since the soup began.
    pub fn protocells_formed(&self) -> u64 {
        self.protocells_formed
    }

    /// Protocells that burst or emptied out.
    pub fn protocells_burst(&self) -> u64 {
        self.protocells_burst
    }

    /// Indices of the chains inside `protocell`.
    pub(super) fn members_of(&self, protocell: &Protocell) -> Vec<usize> {
        (0..self.molecules.len())
            .filter(|&i| {
                let m = &self.molecules[i];
                m.is_amino_acid() && protocell.contains(m.position.x, m.position.y)
            })
            .collect()
    }

    /// Chains that are inside any protocell.
    pub fn chains_in_protocells(&self) -> usize {
        self.molecules
            .iter()
            .filter(|m| m.is_amino_acid() && self.protocells.iter().any(|p| p.contains(m.position.x, m.position.y)))
            .count()
    }

    /// Forms protocells where chains crowd (while dry), and bursts them when the
    /// pool refills or they empty out. Called once per step, after movement.
    pub(super) fn update_protocells(&mut self) {
        if self.wetness >= BURST_ABOVE_WETNESS {
            self.protocells_burst += self.protocells.len() as u64;
            self.protocells.clear();
            return;
        }
        self.settle_protocells();
        let before = self.protocells.len();
        let alive: Vec<Protocell> =
            self.protocells.iter().filter(|p| self.members_of(p).len() >= 2).cloned().collect();
        self.protocells_burst += (before - alive.len()) as u64;
        self.protocells = alive;
        if self.wetness < FORM_BELOW_WETNESS {
            self.form_protocells();
        }
    }

    /// A droplet that is above the water line sinks to it, taking its chains along.
    fn settle_protocells(&mut self) {
        let line = self.water_line();
        for k in 0..self.protocells.len() {
            let p = &self.protocells[k];
            if p.y >= line {
                continue;
            }
            let dy = (line - p.y).min(SETTLE_SPEED);
            for i in self.members_of(&self.protocells[k]) {
                self.molecules[i].position.y += dy;
            }
            self.protocells[k].y += dy;
        }
    }

    fn form_protocells(&mut self) {
        let line = self.water_line();
        // Group the free chains by grid cell. A BTreeMap keeps the order fixed,
        // so a seeded run always forms the same droplets.
        let mut grid: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
        for (i, m) in self.molecules.iter().enumerate() {
            let free = !self.protocells.iter().any(|p| p.contains(m.position.x, m.position.y));
            if m.is_amino_acid() && m.position.y >= line && free {
                let cell = ((m.position.x / GRID_CELL).floor() as i64, (m.position.y / GRID_CELL).floor() as i64);
                grid.entry(cell).or_default().push(i);
            }
        }
        for chains in grid.values().filter(|c| c.len() >= CROWD) {
            if self.protocells.len() >= MAX_PROTOCELLS {
                break;
            }
            let n = chains.len() as f64;
            let x = chains.iter().map(|&i| self.molecules[i].position.x).sum::<f64>() / n;
            let y = chains.iter().map(|&i| self.molecules[i].position.y).sum::<f64>() / n;
            // Droplets do not overlap: skip a crowd that sits next to one already formed.
            if self.protocells.iter().any(|p| (p.x - x).hypot(p.y - y) < p.radius + RADIUS) {
                continue;
            }
            let id = self.next_protocell_id;
            self.next_protocell_id += 1;
            self.protocells_formed += 1;
            self.protocells.push(Protocell { id, x, y, radius: RADIUS, born_step: self.step });
        }
    }

    /// Chains sharing a droplet cross over more often than free ones.
    pub(super) fn recombine_in_protocells(&mut self) {
        for k in 0..self.protocells.len() {
            if self.amino_acid_count() >= MAX_AMINO_ACIDS {
                return;
            }
            let members = self.members_of(&self.protocells[k]);
            if members.len() < 2 || self.rng.random::<f64>() >= SHARED_RECOMBINATION_RATE {
                continue;
            }
            let i = members[self.rng.random_range(0..members.len())];
            let j = members[self.rng.random_range(0..members.len())];
            if i != j {
                self.cross(i, j);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{Cycle, Environment};
    use crate::model::{Molecule, Position};

    fn dry() -> Environment {
        Environment { base_wetness: 0.1, ..Environment::default() }
    }

    fn chain_at(x: f64, y: f64) -> Molecule {
        let mut m = Molecule::amino_acid("ACD".into(), 1.0);
        m.position = Position::new(x, y);
        m
    }

    /// A soup with only the given chains, so a test controls the whole population.
    fn soup_with(chains: Vec<Molecule>) -> PrimordialSoup {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 7);
        soup.molecules = chains;
        soup
    }

    fn crowd() -> Vec<Molecule> {
        vec![chain_at(200.0, 280.0), chain_at(205.0, 282.0), chain_at(198.0, 284.0), chain_at(210.0, 279.0)]
    }

    #[test]
    fn protocell_forms_where_chains_crowd_in_a_dry_pool() {
        let mut soup = soup_with(crowd());
        soup.set_environment(dry());
        soup.simulate_step();
        assert_eq!(soup.protocells.len(), 1);
        assert_eq!(soup.protocells_formed(), 1);
        assert!(soup.chains_in_protocells() >= 2);
    }

    #[test]
    fn no_protocell_when_the_pool_is_wet_or_chains_are_spread_out() {
        let mut wet = soup_with(crowd());
        wet.simulate_step();
        assert!(wet.protocells.is_empty());

        // 50 chains, one per grid cell: the soup is at its cap, so nothing recombines into a crowd.
        let mut spread = PrimordialSoup::with_seed(600.0, 300.0, 300.0, 7.0, 7);
        spread.molecules = (0..50).map(|k| chain_at((k % 10) as f64 * 60.0 + 30.0, (k / 10) as f64 * 60.0 + 30.0)).collect();
        spread.set_environment(dry());
        spread.simulate_step();
        assert!(spread.protocells.is_empty());
    }

    #[test]
    fn protocells_burst_when_the_pool_refills() {
        let mut soup = soup_with(crowd());
        soup.set_environment(dry());
        soup.simulate_step();
        assert_eq!(soup.protocells.len(), 1);
        soup.set_environment(Environment::default());
        soup.simulate_step();
        assert!(soup.protocells.is_empty());
        assert_eq!(soup.protocells_burst(), 1);
    }

    #[test]
    fn chains_cannot_leave_a_protocell_while_it_lasts() {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 1500.0, 7.0, 8);
        soup.molecules = crowd();
        soup.set_environment(Environment { base_temperature: 1500.0, ..dry() });
        soup.simulate_step();
        let cell = soup.protocells[0].clone();
        for _ in 0..300 {
            soup.simulate_step();
            for i in soup.members_of(&cell) {
                assert!(cell.contains(soup.molecules[i].position.x, soup.molecules[i].position.y));
            }
            // The same chains are still there: nobody slipped out.
            assert!(soup.members_of(&cell).len() >= 4, "a chain escaped");
        }
    }

    #[test]
    fn protocells_do_not_overlap() {
        let mut chains = crowd();
        chains.extend([chain_at(215.0, 281.0), chain_at(217.0, 283.0), chain_at(219.0, 280.0)]);
        let mut soup = soup_with(chains);
        soup.set_environment(dry());
        soup.simulate_step();
        let cells = &soup.protocells;
        for (i, a) in cells.iter().enumerate() {
            for b in &cells[i + 1..] {
                assert!((a.x - b.x).hypot(a.y - b.y) >= a.radius + b.radius - 1e-9);
            }
        }
    }

    #[test]
    fn keep_inside_projects_onto_the_boundary() {
        let p = Protocell { id: 1, x: 0.0, y: 0.0, radius: 10.0, born_step: 0 };
        assert_eq!(p.keep_inside(3.0, 4.0), (3.0, 4.0));
        let (x, y) = p.keep_inside(30.0, 40.0);
        assert!(p.contains(x, y) && (x.hypot(y) - 9.8).abs() < 1e-9);
    }

    #[test]
    fn a_dry_pool_gathers_every_molecule_in_the_pool() {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 9);
        soup.add_small_molecules();
        soup.set_environment(Environment { base_wetness: 0.0, ..Environment::default() });
        for _ in 0..400 {
            soup.simulate_step();
        }
        let line = soup.water_line();
        assert!(line > 250.0, "water line {line}");
        assert!(soup.molecules.iter().all(|m| m.position.y >= line - 1.0), "a molecule stayed stranded");
    }

    #[test]
    fn a_wet_soup_has_no_water_line() {
        let soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 10);
        assert_eq!(soup.water_line(), 0.0);
    }

    #[test]
    fn the_environment_drives_the_conditions() {
        let mut soup = PrimordialSoup::with_seed(400.0, 300.0, 300.0, 7.0, 11);
        soup.set_environment(Environment {
            temperature_cycle: Some(Cycle::new(100.0, 30.0)),
            wetness_cycle: Some(Cycle { period: 100.0, amplitude: 0.5, phase: 0.0 }),
            base_wetness: 0.5,
            ..Environment::default()
        });
        for _ in 0..25 {
            soup.simulate_step();
        }
        assert!((soup.temperature - 330.0).abs() < 1e-6);
        assert!((soup.wetness - 1.0).abs() < 1e-6);
        for _ in 0..50 {
            soup.simulate_step();
        }
        assert!((soup.temperature - 270.0).abs() < 1e-6);
    }

    #[test]
    fn protocells_form_in_the_pool_not_in_the_air() {
        // Three chains crowded well above the water line, while the pool is dry.
        let mut soup = soup_with(vec![chain_at(100.0, 40.0), chain_at(104.0, 42.0), chain_at(98.0, 44.0)]);
        soup.set_environment(dry());
        soup.simulate_step();
        assert!(soup.protocells.is_empty());
    }

    #[test]
    fn a_protocell_sinks_with_the_water_line_and_keeps_its_chains() {
        let mut soup = soup_with(crowd());
        soup.set_environment(Environment { base_wetness: 0.4, ..Environment::default() });
        soup.simulate_step();
        assert_eq!(soup.protocells.len(), 1, "no droplet formed at wetness 0.4");
        soup.set_environment(Environment { base_wetness: 0.0, ..Environment::default() });
        for _ in 0..30 {
            soup.simulate_step();
        }
        let cell = &soup.protocells[0];
        assert!(cell.y >= soup.water_line() - 1e-9, "droplet left hanging above the water");
        assert!(soup.members_of(cell).len() >= 3);
    }
}
