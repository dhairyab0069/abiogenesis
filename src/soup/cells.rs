//! Stage 5 in the soup: cells form, take in food, copy their strands, divide,
//! swallow each other, and sometimes stay together. The data and the pure rules
//! are in `cell`; this file is the part that touches the soup.
//!
//! Design choices:
//! - **Off unless asked for.** `enable_cells` switches the layer on; until then
//!   no step here runs and no random number is drawn, so older seeded runs are
//!   unchanged. (The molecule screen's soup has it on.)
//! - **One pass per rule, in a fixed order each step**: form, eat and copy,
//!   share, swallow, die, divide, tidy colonies, move. Each pass is small enough
//!   to read and test on its own.
//! - **Cells are found by id, not by position in the list**, wherever the list
//!   can change in the middle of a pass.
//! - **Bounded cost.** At most [`MAX_CELLS`] cells, each with at most
//!   [`MAX_CELL_STRANDS`] strands, scan the molecule list once a step: a few
//!   thousand distance checks.

use super::PrimordialSoup;
use super::cell::*;
use super::chemistry::{
    COPY_ERROR, MAX_FREE_NUCLEOTIDES, MAX_LIPIDS, MAX_STRANDS, copy_strand, decay_probability,
    replication_rate,
};
use crate::model::{BASES, Molecule, Position};
use rand::Rng;
use std::collections::{HashMap, VecDeque};

/// Chance per step that the soup closes a new cell around a crowd of lipids.
const FORM_RATE: f64 = 0.02;
const FORM_RADIUS: f64 = 60.0;
const FORM_LIPIDS: usize = 3;
const FORM_STRANDS: usize = 3;
/// Strands this close to a lipid crowd are drawn into the new cell.
const FORM_DRAW_RADIUS: f64 = 160.0;
/// Items a cell takes in per step at most, and the chance for each.
const ABSORB_PER_STEP: usize = 2;
const ABSORB_RATE: f64 = 0.3;
/// Chance per step that a cell with strands makes a lipid for its membrane.
const LIPID_SYNTHESIS: f64 = 0.01;
/// Chance per step that a big cell swallows a smaller one it overlaps.
const ENGULF_RATE: f64 = 0.05;
/// The host must be this much wider than the guest.
const ENGULF_RATIO: f64 = 1.12;
/// Chance that a strand passed to a daughter has one wrong base.
const DIVISION_ERROR: f64 = 0.06;
const CELL_SPEED: f64 = 1.0;
/// Events kept for the log.
const EVENT_LOG: usize = 60;

/// Running totals for the stats.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellStats {
    pub formed: u64,
    pub divisions: u64,
    pub deaths: u64,
    pub engulfments: u64,
    pub eukaryotes_formed: u64,
    pub colonies_formed: u64,
}

/// Where the lipids and strands were when a cell closed around them, so the
/// view can replay the formation. Positions are in soup pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct FormationRecord {
    pub cell_id: u64,
    pub step: u64,
    pub centre: (f64, f64),
    pub lipids: Vec<(f64, f64)>,
    pub strands: Vec<(f64, f64)>,
}

/// A swallowing: where the guest was relative to the host, and how big, for the view.
#[derive(Clone, Debug, PartialEq)]
pub struct EngulfRecord {
    pub host_id: u64,
    pub step: u64,
    pub guest_offset: (f64, f64),
    pub guest_radius: f64,
    /// The host had no nucleus before this.
    pub new_nucleus: bool,
}

/// Records kept for the view.
const FORMATION_RECORDS: usize = 16;
const ENGULF_RECORDS: usize = 8;

/// The cell layer's state, kept in one place so the soup has a single field for it.
#[derive(Clone, Debug)]
pub(super) struct CellLayer {
    pub on: bool,
    pub next_cell_id: u64,
    pub next_colony_id: u64,
    pub stats: CellStats,
    pub events: VecDeque<(u64, CellEvent)>,
    pub formations: VecDeque<FormationRecord>,
    pub engulfs: VecDeque<EngulfRecord>,
}

impl Default for CellLayer {
    fn default() -> Self {
        Self {
            on: false,
            next_cell_id: 1,
            next_colony_id: 1,
            stats: CellStats::default(),
            events: VecDeque::new(),
            formations: VecDeque::new(),
            engulfs: VecDeque::new(),
        }
    }
}

impl PrimordialSoup {
    /// Switches the stage 5 cell layer on.
    pub fn enable_cells(&mut self) {
        self.cell_layer.on = true;
    }

    pub fn cells_enabled(&self) -> bool {
        self.cell_layer.on
    }

    /// Adds `count` ready-made cells (three strands, some membrane and a few
    /// nucleotides each) at random places, for the demo and for tests.
    pub fn seed_cells(&mut self, count: usize) {
        self.cell_layer.on = true;
        for _ in 0..count {
            if self.cells.len() >= MAX_CELLS {
                break;
            }
            let at = self.random_position();
            let mut cell = self.new_cell(at.x, at.y);
            for _ in 0..3 {
                let len = self.rng.random_range(8..=12);
                let seq: String = (0..len)
                    .map(|_| BASES[self.rng.random_range(0..BASES.len())])
                    .collect();
                cell.strands.push(Molecule::strand(seq));
            }
            cell.lipids = 6;
            cell.nucleotides = 10;
            self.cells.push(cell);
        }
    }

    pub fn cell_stats(&self) -> &CellStats {
        &self.cell_layer.stats
    }

    /// The most recent cell events, oldest first.
    pub fn cell_events(&self) -> &VecDeque<(u64, CellEvent)> {
        &self.cell_layer.events
    }

    /// How recent cells formed (for replaying the formation), oldest first.
    pub fn formation_records(&self) -> &VecDeque<FormationRecord> {
        &self.cell_layer.formations
    }

    /// How recent swallowings went (for replaying them), oldest first.
    pub fn engulf_records(&self) -> &VecDeque<EngulfRecord> {
        &self.cell_layer.engulfs
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn eukaryote_count(&self) -> usize {
        self.cells.iter().filter(|c| c.is_eukaryote()).count()
    }

    pub fn mitochondria_total(&self) -> u32 {
        self.cells.iter().map(|c| c.mitochondria).sum()
    }

    /// Colonies with at least two members.
    pub fn colony_count(&self) -> usize {
        self.colony_sizes().len()
    }

    pub fn largest_colony(&self) -> usize {
        self.colony_sizes().values().copied().max().unwrap_or(0)
    }

    /// The average cell fitness, or 0 with no cells.
    pub fn mean_cell_fitness(&self) -> f64 {
        if self.cells.is_empty() {
            0.0
        } else {
            self.cells.iter().map(Cell::fitness).sum::<f64>() / self.cells.len() as f64
        }
    }

    /// Members of each colony that has any.
    fn colony_sizes(&self) -> HashMap<u64, usize> {
        let mut sizes = HashMap::new();
        for c in &self.cells {
            if let Some(id) = c.colony {
                *sizes.entry(id).or_insert(0) += 1;
            }
        }
        sizes.retain(|_, n| *n >= 2);
        sizes
    }

    /// The division-of-labour bonus of every colony.
    fn labour(&self) -> HashMap<u64, f64> {
        let mut roles: HashMap<u64, Vec<Role>> = HashMap::new();
        for c in &self.cells {
            if let Some(id) = c.colony {
                let seen = roles.entry(id).or_default();
                if c.role != Role::Generalist && !seen.contains(&c.role) {
                    seen.push(c.role);
                }
            }
        }
        roles
            .into_iter()
            .map(|(id, r)| (id, labour_bonus(r.len())))
            .collect()
    }

    fn new_cell(&mut self, x: f64, y: f64) -> Cell {
        let id = self.cell_layer.next_cell_id;
        self.cell_layer.next_cell_id += 1;
        Cell::new(id, x, y, self.step)
    }

    fn log_cell(&mut self, event: CellEvent) {
        let events = &mut self.cell_layer.events;
        if events.len() >= EVENT_LOG {
            events.pop_front();
        }
        events.push_back((self.step, event));
    }

    /// One step of the cell layer. Does nothing unless it was switched on.
    pub(super) fn step_cells(&mut self) {
        if !self.cell_layer.on {
            return;
        }
        self.form_cell();
        for i in 0..self.cells.len() {
            self.absorb(i);
            self.cell_chemistry(i);
        }
        self.share_food();
        self.engulf();
        self.cells_die();
        self.divide_ready();
        self.tidy_colonies();
        self.move_cells();
    }

    /// A crowd of lipids closes up into a cell and draws in the nearest strands.
    pub(super) fn form_cell(&mut self) {
        if self.cells.len() >= MAX_CELLS || self.rng.random::<f64>() >= FORM_RATE {
            return;
        }
        let lipids: Vec<usize> = (0..self.molecules.len())
            .filter(|&i| self.molecules[i].is_lipid())
            .collect();
        if lipids.len() < FORM_LIPIDS {
            return;
        }
        let here = self.molecules[lipids[self.rng.random_range(0..lipids.len())]].position;
        let near = |m: &Molecule| m.position.distance_to(&here) <= FORM_RADIUS;
        let lipids: Vec<usize> = lipids
            .into_iter()
            .filter(|&i| near(&self.molecules[i]))
            .collect();
        let mut strands: Vec<(usize, f64)> = (0..self.molecules.len())
            .filter(|&i| self.molecules[i].is_strand())
            .map(|i| (i, self.molecules[i].position.distance_to(&here)))
            .filter(|&(_, d)| d <= FORM_DRAW_RADIUS)
            .collect();
        strands.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        let strands: Vec<usize> = strands
            .into_iter()
            .take(FORM_STRANDS)
            .map(|(i, _)| i)
            .collect();
        if lipids.len() < FORM_LIPIDS || strands.is_empty() {
            return;
        }
        let mut cell = self.new_cell(here.x, here.y);
        cell.lipids = lipids.len().min(LIPID_CAP as usize) as u32;
        let mut taken: Vec<usize> = lipids.into_iter().take(cell.lipids as usize).collect();
        taken.extend(&strands);
        cell.strands = strands.iter().map(|&i| self.molecules[i].clone()).collect();
        let at = |i: &usize| (self.molecules[*i].position.x, self.molecules[*i].position.y);
        let record = FormationRecord {
            cell_id: cell.id,
            step: self.step,
            centre: (here.x, here.y),
            lipids: taken.iter().take(cell.lipids as usize).map(at).collect(),
            strands: strands.iter().map(at).collect(),
        };
        if self.cell_layer.formations.len() >= FORMATION_RECORDS {
            self.cell_layer.formations.pop_front();
        }
        self.cell_layer.formations.push_back(record);
        taken.sort_unstable_by(|a, b| b.cmp(a));
        for i in taken {
            self.molecules.swap_remove(i);
        }
        self.cell_layer.stats.formed += 1;
        self.log_cell(CellEvent::Formed { id: cell.id });
        self.cells.push(cell);
    }

    /// The cell takes in a couple of nearby lipids, nucleotides or strands per
    /// step (strands first), as far as it has room.
    pub(super) fn absorb(&mut self, i: usize) {
        let cell = &self.cells[i];
        let (at, reach) = (Position::new(cell.x, cell.y), cell.reach());
        let priority = |m: &Molecule| {
            if m.is_strand() {
                0
            } else if m.is_lipid() {
                1
            } else {
                2
            }
        };
        let mut near: Vec<(usize, u8, f64)> = self
            .molecules
            .iter()
            .enumerate()
            .filter(|(_, m)| m.is_strand() || m.is_lipid() || m.is_nucleotide())
            .map(|(j, m)| (j, priority(m), m.position.distance_to(&at)))
            .filter(|&(_, _, d)| d <= reach)
            .collect();
        near.sort_by(|a, b| a.1.cmp(&b.1).then(a.2.total_cmp(&b.2)).then(a.0.cmp(&b.0)));
        let mut taken = Vec::new();
        for (j, _, _) in near {
            if taken.len() >= ABSORB_PER_STEP {
                break;
            }
            let cell = &self.cells[i];
            let m = &self.molecules[j];
            let room = if m.is_strand() {
                cell.strands.len() < MAX_CELL_STRANDS
            } else if m.is_lipid() {
                cell.lipids < LIPID_CAP
            } else {
                cell.nucleotides < NUCLEOTIDE_CAP
            };
            if !room || self.rng.random::<f64>() >= ABSORB_RATE {
                continue;
            }
            let cell = &mut self.cells[i];
            if m.is_strand() {
                cell.strands.push(m.clone());
            } else if m.is_lipid() {
                cell.lipids += 1;
            } else {
                cell.nucleotides += 1;
            }
            taken.push(j);
        }
        taken.sort_unstable_by(|a, b| b.cmp(a));
        for j in taken {
            self.molecules.swap_remove(j);
        }
    }

    /// Inside one cell: it makes some membrane, its strands copy themselves from
    /// its stock of nucleotides (a nucleus lowers the error, mitochondria speed it
    /// up) and fall apart now and then (a nucleus slows that).
    pub(super) fn cell_chemistry(&mut self, i: usize) {
        let wetness = self.wetness;
        let Self { cells, rng, .. } = self;
        let cell = &mut cells[i];
        if !cell.strands.is_empty()
            && cell.lipids < LIPID_CAP
            && rng.random::<f64>() < LIPID_SYNTHESIS * cell.energy()
        {
            cell.lipids += 1;
        }
        let (energy, error) = (cell.energy(), COPY_ERROR * cell.error_scale());
        for k in 0..cell.strands.len() {
            let Some(seq) = cell.strands[k].strand_sequence().map(str::to_string) else {
                continue;
            };
            if cell.strands.len() >= MAX_CELL_STRANDS || cell.nucleotides < seq.len() as u32 {
                continue;
            }
            if rng.random::<f64>() < replication_rate(&seq, true) * energy {
                cell.nucleotides -= seq.len() as u32;
                let copy = Molecule::strand(copy_strand(rng, &seq, error));
                cell.strands.push(copy);
            }
        }
        let scale = cell.decay_scale();
        cell.strands.retain(|m| {
            let p = decay_probability(m, wetness, true) * scale;
            p <= 0.0 || rng.random::<f64>() >= p
        });
    }

    /// Feeders hand a nucleotide to the colony mate that has the fewest.
    pub(super) fn share_food(&mut self) {
        for f in 0..self.cells.len() {
            let giver = &self.cells[f];
            let (Some(colony), Role::Feeder) = (giver.colony, giver.role) else {
                continue;
            };
            if giver.nucleotides < 4 {
                continue;
            }
            let neediest = (0..self.cells.len())
                .filter(|&j| {
                    j != f
                        && self.cells[j].colony == Some(colony)
                        && self.cells[j].nucleotides < NUCLEOTIDE_CAP
                })
                .min_by_key(|&j| (self.cells[j].nucleotides, j));
            if let Some(j) =
                neediest.filter(|&j| self.cells[j].nucleotides + 1 < self.cells[f].nucleotides)
            {
                self.cells[f].nucleotides -= 1;
                self.cells[j].nucleotides += 1;
            }
        }
    }

    /// A big cell that overlaps a much smaller free cell may swallow it. The guest
    /// becomes a mitochondrion and the host's strands get a nucleus.
    pub(super) fn engulf(&mut self) {
        for h in 0..self.cells.len() {
            for g in 0..self.cells.len() {
                if h == g || !can_engulf(&self.cells[h], &self.cells[g]) {
                    continue;
                }
                if self.rng.random::<f64>() < ENGULF_RATE {
                    self.swallow(h, g);
                    return;
                }
            }
        }
    }

    /// Cell `g` becomes a mitochondrion of cell `h`; its strands move into the
    /// host's nucleus (what does not fit is released into the soup).
    pub(super) fn swallow(&mut self, h: usize, g: usize) {
        let (offset, guest_radius) = (
            (
                self.cells[g].x - self.cells[h].x,
                self.cells[g].y - self.cells[h].y,
            ),
            self.cells[g].radius(),
        );
        let guest = self.cells.remove(g);
        let h = if g < h { h - 1 } else { h };
        let record = EngulfRecord {
            host_id: self.cells[h].id,
            step: self.step,
            guest_offset: offset,
            guest_radius,
            new_nucleus: !self.cells[h].nucleus,
        };
        if self.cell_layer.engulfs.len() >= ENGULF_RECORDS {
            self.cell_layer.engulfs.pop_front();
        }
        self.cell_layer.engulfs.push_back(record);
        let (host_id, guest_id) = (self.cells[h].id, guest.id);
        let host = &mut self.cells[h];
        let was_eukaryote = host.is_eukaryote();
        host.mitochondria += 1;
        host.nucleus = true;
        host.lipids = (host.lipids + guest.lipids).min(LIPID_CAP);
        host.nucleotides = (host.nucleotides + guest.nucleotides).min(NUCLEOTIDE_CAP);
        let room = MAX_CELL_STRANDS - host.strands.len().min(MAX_CELL_STRANDS);
        let (host_pos, mut overflow) = (Position::new(host.x, host.y), Vec::new());
        for (k, s) in guest.strands.into_iter().enumerate() {
            if k < room {
                host.strands.push(s)
            } else {
                overflow.push(s)
            }
        }
        let now_eukaryote = host.is_eukaryote();
        for s in overflow {
            self.release_strand(s, host_pos);
        }
        self.cell_layer.stats.engulfments += 1;
        self.log_cell(CellEvent::Engulfed {
            host: host_id,
            guest: guest_id,
        });
        if now_eukaryote && !was_eukaryote {
            self.cell_layer.stats.eukaryotes_formed += 1;
            self.log_cell(CellEvent::BecameEukaryote { id: host_id });
        }
    }

    /// Puts a strand back into the soup near `at`, if the soup has room for it.
    fn release_strand(&mut self, mut strand: Molecule, at: Position) {
        if self.strand_count() < MAX_STRANDS {
            strand.position = self.near(at, 20.0);
            self.molecules.push(strand);
        }
    }

    fn near(&mut self, at: Position, spread: f64) -> Position {
        Position::new(
            (at.x + self.rng.random_range(-spread..spread)).clamp(0.0, self.width),
            (at.y + self.rng.random_range(-spread..spread)).clamp(0.0, self.height),
        )
    }

    /// Cells die by chance (see `Cell::death_rate`) and give their contents back.
    pub(super) fn cells_die(&mut self) {
        let labour = self.labour();
        let mut dead = Vec::new();
        for i in 0..self.cells.len() {
            let l = self.cells[i]
                .colony
                .and_then(|c| labour.get(&c))
                .copied()
                .unwrap_or(1.0);
            if self.rng.random::<f64>() < self.cells[i].death_rate(l) {
                dead.push(i);
            }
        }
        for i in dead.into_iter().rev() {
            let cell = self.cells.remove(i);
            self.spill(cell);
        }
    }

    /// A dead cell's strands, membrane and nucleotides go back into the soup, as far as it has room.
    pub(super) fn spill(&mut self, cell: Cell) {
        let at = Position::new(cell.x, cell.y);
        for s in cell.strands {
            self.release_strand(s, at);
        }
        for _ in 0..cell.lipids {
            if self.lipid_count() < MAX_LIPIDS {
                let mut l = Molecule::lipid();
                l.position = self.near(at, 25.0);
                self.molecules.push(l);
            }
        }
        for _ in 0..cell.nucleotides {
            if self.nucleotide_count() < MAX_FREE_NUCLEOTIDES {
                let mut n = Molecule::nucleotide(BASES[self.rng.random_range(0..BASES.len())]);
                n.position = self.near(at, 25.0);
                self.molecules.push(n);
            }
        }
        self.cell_layer.stats.deaths += 1;
        self.log_cell(CellEvent::Died { id: cell.id });
    }

    /// Cells with enough contents divide by chance. A eukaryote's daughter may stay attached.
    pub(super) fn divide_ready(&mut self) {
        let labour = self.labour();
        let sizes = self.colony_sizes();
        let ready: Vec<u64> = self
            .cells
            .iter()
            .filter(|c| c.division_ready())
            .map(|c| c.id)
            .collect();
        for id in ready {
            if self.cells.len() >= MAX_CELLS {
                return;
            }
            let Some(i) = self.cells.iter().position(|c| c.id == id) else {
                continue;
            };
            let c = &self.cells[i];
            let l = c
                .colony
                .and_then(|k| labour.get(&k))
                .copied()
                .unwrap_or(1.0);
            if self.rng.random::<f64>() >= c.division_rate(l) {
                continue;
            }
            let room = c
                .colony
                .and_then(|k| sizes.get(&k))
                .is_none_or(|&n| n < MAX_COLONY_SIZE);
            let attach = c.is_eukaryote()
                && room
                && self.rng.random::<f64>() < stick_probability(c.colony.is_some());
            self.divide(i, attach);
        }
    }

    /// Splits cell `i` in two: strands go alternately to the mother and the
    /// daughter (each with a small chance of one wrong base), the membrane and
    /// the nucleotides are shared out, and the nucleus and mitochondria are
    /// duplicated. With `attach` the daughter touches the mother and joins her colony.
    /// Returns the daughter's index, or `None` at the cell cap.
    pub(super) fn divide(&mut self, i: usize, attach: bool) -> Option<usize> {
        if self.cells.len() >= MAX_CELLS {
            return None;
        }
        let (mx, my, mother_radius) = (self.cells[i].x, self.cells[i].y, self.cells[i].radius());
        let error = DIVISION_ERROR * self.cells[i].error_scale();
        let strands = std::mem::take(&mut self.cells[i].strands);
        let (mut keep, mut give) = (Vec::new(), Vec::new());
        for (k, s) in strands.into_iter().enumerate() {
            let s = self.maybe_miscopy(s, error);
            if k % 2 == 0 {
                keep.push(s)
            } else {
                give.push(s)
            }
        }
        let mut daughter = self.new_cell(mx, my);
        let mother = &mut self.cells[i];
        daughter.parent = Some(mother.id);
        daughter.generation = mother.generation + 1;
        daughter.strands = give;
        mother.strands = keep;
        daughter.lipids = mother.lipids / 2;
        mother.lipids -= daughter.lipids;
        daughter.nucleotides = mother.nucleotides / 2;
        mother.nucleotides -= daughter.nucleotides;
        daughter.nucleus = mother.nucleus;
        daughter.mitochondria = mother.mitochondria;
        let (mother_id, daughter_id) = (mother.id, daughter.id);
        let gap = mother_radius + daughter.radius() - if attach { 4.0 } else { -6.0 };
        let angle = self.rng.random_range(0.0..std::f64::consts::TAU);
        let r = daughter
            .radius()
            .min(self.width / 2.0)
            .min(self.height / 2.0);
        daughter.x = (mx + gap * angle.cos()).clamp(r, self.width - r);
        daughter.y = (my + gap * angle.sin()).clamp(r, self.height - r);
        if attach {
            let colony = match self.cells[i].colony {
                Some(c) => c,
                None => {
                    let c = self.cell_layer.next_colony_id;
                    self.cell_layer.next_colony_id += 1;
                    self.cells[i].colony = Some(c);
                    self.cells[i].role = Role::Germ;
                    self.cell_layer.stats.colonies_formed += 1;
                    self.log_cell(CellEvent::ColonyFormed {
                        colony: c,
                        founder: mother_id,
                    });
                    c
                }
            };
            let count = |role: Role| {
                self.cells
                    .iter()
                    .filter(|c| c.colony == Some(colony) && c.role == role)
                    .count()
            };
            daughter.colony = Some(colony);
            daughter.role = next_role(count(Role::Germ), count(Role::Soma), count(Role::Feeder));
        }
        self.cell_layer.stats.divisions += 1;
        self.log_cell(CellEvent::Divided {
            mother: mother_id,
            daughter: daughter_id,
            attached: attach,
        });
        self.cells.push(daughter);
        Some(self.cells.len() - 1)
    }

    /// With probability `error`, one base of the strand is replaced by another.
    fn maybe_miscopy(&mut self, strand: Molecule, error: f64) -> Molecule {
        let Some(seq) = strand.strand_sequence().filter(|s| !s.is_empty()) else {
            return strand;
        };
        if self.rng.random::<f64>() >= error {
            return strand;
        }
        let mut bases: Vec<char> = seq.chars().collect();
        let at = self.rng.random_range(0..bases.len());
        let old = bases[at];
        let choices: Vec<char> = BASES.iter().copied().filter(|&b| b != old).collect();
        bases[at] = choices[self.rng.random_range(0..choices.len())];
        Molecule::strand(bases.into_iter().collect())
    }

    /// A colony left with one member is just a cell again.
    pub(super) fn tidy_colonies(&mut self) {
        let sizes = self.colony_sizes();
        for c in &mut self.cells {
            if c.colony.is_some_and(|k| !sizes.contains_key(&k)) {
                c.colony = None;
                c.role = Role::Generalist;
            }
        }
    }

    /// Cells drift a little (slower than molecules); the members of a colony move as one body.
    pub(super) fn move_cells(&mut self) {
        let scale = CELL_SPEED * self.temperature / 300.0;
        let (w, h) = (self.width, self.height);
        let mut moved: Vec<u64> = Vec::new();
        for i in 0..self.cells.len() {
            let (dx, dy) = (
                (self.rng.random::<f64>() - 0.5) * scale,
                (self.rng.random::<f64>() - 0.5) * scale,
            );
            match self.cells[i].colony {
                None => {
                    let r = self.cells[i].radius().min(w / 2.0).min(h / 2.0);
                    let c = &mut self.cells[i];
                    c.x = (c.x + dx).clamp(r, w - r);
                    c.y = (c.y + dy).clamp(r, h - r);
                }
                Some(k) if !moved.contains(&k) => {
                    moved.push(k);
                    let members: Vec<usize> = (0..self.cells.len())
                        .filter(|&j| self.cells[j].colony == Some(k))
                        .collect();
                    let dx = clamp_shift(
                        dx,
                        members
                            .iter()
                            .map(|&j| (self.cells[j].x, self.cells[j].radius())),
                        w,
                    );
                    let dy = clamp_shift(
                        dy,
                        members
                            .iter()
                            .map(|&j| (self.cells[j].y, self.cells[j].radius())),
                        h,
                    );
                    for j in members {
                        self.cells[j].x += dx;
                        self.cells[j].y += dy;
                    }
                }
                Some(_) => {}
            }
        }
    }
}

/// Whether `host` may swallow `guest`: both free, the host clearly bigger, the
/// guest not already a eukaryote, the host with room for another mitochondrion
/// and the two overlapping.
pub(super) fn can_engulf(host: &Cell, guest: &Cell) -> bool {
    host.colony.is_none()
        && guest.colony.is_none()
        && !guest.is_eukaryote()
        && host.mitochondria < MAX_MITOCHONDRIA
        && host.radius() >= guest.radius() * ENGULF_RATIO
        && (host.x - guest.x).hypot(host.y - guest.y) < host.radius() + guest.radius() * 0.5
}

/// The shift to apply to every coordinate of a body so that none leaves `0..limit`.
/// `shift` itself when it fits, else as much of it as does (0 when the body is wider than the world).
fn clamp_shift(shift: f64, members: impl Iterator<Item = (f64, f64)> + Clone, limit: f64) -> f64 {
    let lo = members
        .clone()
        .map(|(p, r)| r - p)
        .fold(f64::NEG_INFINITY, f64::max);
    let hi = members
        .map(|(p, r)| limit - r - p)
        .fold(f64::INFINITY, f64::min);
    if lo > hi {
        0.0
    } else {
        shift.clamp(lo.min(0.0), hi.max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{Environment, Preset};

    fn at(mut m: Molecule, x: f64, y: f64) -> Molecule {
        m.position = Position::new(x, y);
        m
    }

    fn strand(seq: &str) -> Molecule {
        Molecule::strand(seq.into())
    }

    /// An empty, dry, chemistry-free soup with the cell layer on.
    fn bare() -> PrimordialSoup {
        let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 61);
        soup.molecules.clear();
        soup.set_environment(Environment {
            base_wetness: 0.0,
            ..Environment::default()
        });
        soup.enable_cells();
        soup
    }

    fn ready_cell(soup: &mut PrimordialSoup, x: f64, y: f64) -> usize {
        let mut cell = soup.new_cell(x, y);
        cell.strands = (0..6).map(|_| strand("GAUCAUGC")).collect();
        cell.lipids = 10;
        cell.nucleotides = 8;
        soup.cells.push(cell);
        soup.cells.len() - 1
    }

    fn total_strands(soup: &PrimordialSoup) -> usize {
        soup.molecules.iter().filter(|m| m.is_strand()).count()
            + soup.cells.iter().map(|c| c.strands.len()).sum::<usize>()
    }

    #[test]
    fn a_lipid_crowd_closes_into_a_cell_and_draws_in_strands() {
        let mut soup = bare();
        soup.molecules = vec![
            at(Molecule::lipid(), 200.0, 200.0),
            at(Molecule::lipid(), 215.0, 205.0),
            at(Molecule::lipid(), 205.0, 220.0),
            at(strand("GGUCAUAC"), 260.0, 230.0),
            at(strand("GAUCAUGC"), 700.0, 500.0),
        ];
        for _ in 0..2_000 {
            soup.form_cell();
            if !soup.cells.is_empty() {
                break;
            }
        }
        assert_eq!(soup.cells.len(), 1);
        let cell = &soup.cells[0];
        let record = soup
            .formation_records()
            .back()
            .expect("the formation was recorded");
        assert_eq!(
            (record.cell_id, record.lipids.len(), record.strands.len()),
            (cell.id, 3, 1)
        );
        assert!(
            record.lipids.contains(&(200.0, 200.0)) && record.strands == vec![(260.0, 230.0)],
            "where they were"
        );
        assert_eq!(
            (cell.lipids, cell.strands.len()),
            (3, 1),
            "the near strand came in, the far one did not"
        );
        assert_eq!(
            soup.molecules.len(),
            1,
            "everything the cell holds left the soup"
        );
        assert_eq!(soup.cell_stats().formed, 1);
        assert!(matches!(
            soup.cell_events().back(),
            Some((_, CellEvent::Formed { .. }))
        ));
    }

    #[test]
    fn no_strands_nearby_means_no_cell() {
        let mut soup = bare();
        soup.molecules = (0..4)
            .map(|k| at(Molecule::lipid(), 200.0 + k as f64 * 5.0, 200.0))
            .collect();
        for _ in 0..2_000 {
            soup.form_cell();
        }
        assert!(soup.cells.is_empty());
    }

    #[test]
    fn a_cell_takes_in_nearby_food_and_ignores_the_far() {
        let mut soup = bare();
        let i = {
            let cell = soup.new_cell(300.0, 300.0);
            soup.cells.push(cell);
            0
        };
        soup.molecules = vec![
            at(Molecule::lipid(), 310.0, 300.0),
            at(Molecule::nucleotide('A'), 295.0, 305.0),
            at(strand("GGUCAUAC"), 305.0, 295.0),
            at(Molecule::lipid(), 600.0, 500.0),
        ];
        for _ in 0..300 {
            soup.absorb(i);
        }
        let cell = &soup.cells[0];
        assert_eq!(
            (cell.lipids, cell.nucleotides, cell.strands.len()),
            (1, 1, 1)
        );
        assert_eq!(soup.molecules.len(), 1);
    }

    #[test]
    fn a_full_cell_takes_in_no_more() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i]
            .strands
            .resize(MAX_CELL_STRANDS, strand("GAUCAUGC"));
        soup.cells[i].lipids = LIPID_CAP;
        soup.molecules = vec![
            at(strand("AAAAAA"), 300.0, 300.0),
            at(Molecule::lipid(), 301.0, 300.0),
        ];
        for _ in 0..300 {
            soup.absorb(i);
        }
        assert_eq!(
            (soup.cells[i].strands.len(), soup.cells[i].lipids),
            (MAX_CELL_STRANDS, LIPID_CAP)
        );
        assert_eq!(soup.molecules.len(), 2);
    }

    #[test]
    fn strands_in_a_cell_copy_themselves_using_its_nucleotides() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].strands = vec![strand("GGUCAUAC")];
        soup.cells[i].nucleotides = 8;
        soup.cells[i].lipids = LIPID_CAP;
        for _ in 0..5_000 {
            soup.cell_chemistry(i);
            if soup.cells[i].strands.len() > 1 {
                break;
            }
        }
        assert!(soup.cells[i].strands.len() >= 2);
        assert_eq!(
            soup.cells[i].nucleotides, 0,
            "the copy used up eight nucleotides"
        );
    }

    #[test]
    fn without_nucleotides_nothing_copies() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].nucleotides = 0;
        let before = soup.cells[i].strands.len();
        for _ in 0..3_000 {
            soup.cell_chemistry(i);
            soup.cells[i].strands.truncate(before);
        }
        assert!(soup.cells[i].strands.len() <= before);
        assert_eq!(soup.cells[i].nucleotides, 0);
    }

    #[test]
    fn a_nucleus_keeps_strands_from_falling_apart() {
        let survivors = |nucleus: bool| {
            let mut soup = bare();
            soup.set_environment(Environment::default());
            let i = ready_cell(&mut soup, 300.0, 300.0);
            soup.cells[i].strands = (0..MAX_CELL_STRANDS).map(|_| strand("AAAAAAAA")).collect();
            soup.cells[i].nucleotides = 0;
            soup.cells[i].nucleus = nucleus;
            for _ in 0..1_500 {
                soup.cell_chemistry(i);
            }
            soup.cells[i].strands.len()
        };
        assert!(survivors(true) > survivors(false));
    }

    #[test]
    fn dividing_shares_the_contents_and_keeps_the_family_tree() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        let (strands, lipids, nucleotides) = (
            soup.cells[i].strands.len(),
            soup.cells[i].lipids,
            soup.cells[i].nucleotides,
        );
        let d = soup.divide(i, false).expect("room for a daughter");
        let (mother, daughter) = (&soup.cells[i], &soup.cells[d]);
        assert_eq!(mother.strands.len() + daughter.strands.len(), strands);
        assert_eq!(mother.lipids + daughter.lipids, lipids);
        assert_eq!(mother.nucleotides + daughter.nucleotides, nucleotides);
        assert_eq!((daughter.parent, daughter.generation), (Some(mother.id), 1));
        assert!(daughter.colony.is_none() && mother.colony.is_none());
        assert!(
            mother
                .strands
                .iter()
                .chain(&daughter.strands)
                .all(|s| s.strand_sequence().is_some_and(|q| q.len() == 8))
        );
        assert_eq!(soup.cell_stats().divisions, 1);
        assert!(matches!(
            soup.cell_events().back(),
            Some((
                _,
                CellEvent::Divided {
                    attached: false,
                    ..
                }
            ))
        ));
    }

    #[test]
    fn daughters_inherit_the_odd_copying_error() {
        let mut changed = 0;
        let mut total = 0;
        for seed in 0..40 {
            let mut soup = bare();
            soup.rng = rand::SeedableRng::seed_from_u64(seed);
            let i = ready_cell(&mut soup, 300.0, 300.0);
            let d = soup.divide(i, false).unwrap();
            for s in soup.cells[i].strands.iter().chain(&soup.cells[d].strands) {
                total += 1;
                changed += (s.strand_sequence() != Some("GAUCAUGC")) as usize;
            }
        }
        assert!(
            changed > 0 && changed * 4 < total,
            "{changed} of {total} strands changed"
        );
    }

    #[test]
    fn a_cell_with_too_little_does_not_divide() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].lipids = 2;
        for _ in 0..3_000 {
            soup.divide_ready();
        }
        assert_eq!(soup.cells.len(), 1);
        soup.cells[i].lipids = DIVIDE_LIPIDS;
        for _ in 0..3_000 {
            soup.divide_ready();
            if soup.cells.len() > 1 {
                break;
            }
        }
        assert!(soup.cells.len() > 1, "a ready cell divides in the end");
    }

    #[test]
    fn division_stops_at_the_cell_cap() {
        let mut soup = bare();
        for k in 0..MAX_CELLS {
            ready_cell(
                &mut soup,
                30.0 + (k % 12) as f64 * 60.0,
                60.0 + (k / 12) as f64 * 100.0,
            );
        }
        assert!(soup.divide(0, false).is_none());
        for _ in 0..500 {
            soup.divide_ready();
        }
        assert_eq!(soup.cells.len(), MAX_CELLS);
    }

    #[test]
    fn a_big_cell_swallows_a_small_one_and_gets_a_mitochondrion_and_a_nucleus() {
        let mut soup = bare();
        let host = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[host].lipids = 12;
        let guest = {
            let mut c = soup.new_cell(310.0, 305.0);
            c.strands = vec![strand("GGUCAUAC")];
            c.lipids = 2;
            soup.cells.push(c);
            1
        };
        assert!(can_engulf(&soup.cells[host], &soup.cells[guest]));
        assert!(
            !can_engulf(&soup.cells[guest], &soup.cells[host]),
            "the small cell cannot swallow the big one"
        );
        let (guest_id, host_strands) = (soup.cells[guest].id, soup.cells[host].strands.len());
        for _ in 0..2_000 {
            soup.engulf();
            if soup.cells.len() == 1 {
                break;
            }
        }
        assert_eq!(soup.cells.len(), 1);
        let h = &soup.cells[0];
        assert!(h.mitochondria == 1 && h.nucleus && h.is_eukaryote());
        assert_eq!(
            h.strands.len(),
            host_strands + 1,
            "the guest's strand moved into the nucleus"
        );
        assert_eq!(soup.cell_stats().engulfments, 1);
        let rec = soup.engulf_records().back().expect("recorded");
        assert_eq!(
            (rec.host_id, rec.guest_offset, rec.new_nucleus),
            (soup.cells[0].id, (10.0, 5.0), true)
        );
        assert!(rec.guest_radius > 0.0 && soup.engulf_records().len() == 1);
        assert_eq!(soup.cell_stats().eukaryotes_formed, 1);
        assert!(
            soup.cell_events()
                .iter()
                .any(|(_, e)| matches!(e, CellEvent::Engulfed { guest: g, .. } if *g == guest_id))
        );
        assert!(
            soup.cell_events()
                .iter()
                .any(|(_, e)| matches!(e, CellEvent::BecameEukaryote { .. }))
        );
    }

    #[test]
    fn similar_sized_far_apart_or_eukaryote_cells_are_left_alone() {
        let mut soup = bare();
        let a = ready_cell(&mut soup, 300.0, 300.0);
        let b = ready_cell(&mut soup, 305.0, 300.0);
        assert!(!can_engulf(&soup.cells[a], &soup.cells[b]), "same size");
        soup.cells[a].lipids = 14;
        soup.cells[b].lipids = 0;
        soup.cells[b].strands.truncate(1);
        assert!(can_engulf(&soup.cells[a], &soup.cells[b]));
        soup.cells[b].x = 600.0;
        assert!(!can_engulf(&soup.cells[a], &soup.cells[b]), "too far");
        soup.cells[b].x = 305.0;
        soup.cells[b].nucleus = true;
        soup.cells[b].mitochondria = 1;
        assert!(
            !can_engulf(&soup.cells[a], &soup.cells[b]),
            "a eukaryote is not food"
        );
        soup.cells[b].nucleus = false;
        soup.cells[b].mitochondria = 0;
        soup.cells[a].mitochondria = MAX_MITOCHONDRIA;
        assert!(
            !can_engulf(&soup.cells[a], &soup.cells[b]),
            "the host has enough mitochondria"
        );
    }

    #[test]
    fn swallowing_releases_strands_that_do_not_fit() {
        let mut soup = bare();
        let host = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[host]
            .strands
            .resize(MAX_CELL_STRANDS - 1, strand("GAUCAUGC"));
        let mut guest = soup.new_cell(305.0, 300.0);
        guest.strands = vec![strand("GGUCAUAC"), strand("GGUCAUAA"), strand("GGUCAUAG")];
        soup.cells.push(guest);
        soup.swallow(host, 1);
        assert_eq!(soup.cells[0].strands.len(), MAX_CELL_STRANDS);
        assert_eq!(soup.strand_count(), 2, "two strands went back to the soup");
    }

    #[test]
    fn a_eukaryote_daughter_can_stay_attached_and_the_colony_takes_roles() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].nucleus = true;
        soup.cells[i].mitochondria = 1;
        let d = soup.divide(i, true).unwrap();
        let colony = soup.cells[i].colony.expect("a colony began");
        assert_eq!(soup.cells[d].colony, Some(colony));
        assert_eq!(
            (soup.cells[i].role, soup.cells[d].role),
            (Role::Germ, Role::Soma)
        );
        let gap = (soup.cells[i].x - soup.cells[d].x).hypot(soup.cells[i].y - soup.cells[d].y);
        assert!(
            gap <= soup.cells[i].radius() + soup.cells[d].radius(),
            "touching"
        );
        assert!(soup.cells[d].is_eukaryote());
        assert_eq!(soup.colony_count(), 1);
        assert_eq!(soup.cell_stats().colonies_formed, 1);
        // the germ cell divides again: the new member is a feeder, and the colony has three roles
        let again = ready_cell_refill(&mut soup, i);
        let e = soup.divide(again, true).unwrap();
        assert_eq!(soup.cells[e].role, Role::Feeder);
        assert_eq!(soup.labour().get(&colony).copied(), Some(labour_bonus(3)));
        assert_eq!(soup.largest_colony(), 3);
        assert_eq!(soup.cell_stats().colonies_formed, 1, "same colony");
    }

    fn ready_cell_refill(soup: &mut PrimordialSoup, i: usize) -> usize {
        soup.cells[i].strands = (0..6).map(|_| strand("GAUCAUGC")).collect();
        soup.cells[i].lipids = 10;
        i
    }

    #[test]
    fn plain_cells_never_attach_on_their_own() {
        let mut soup = bare();
        for _ in 0..300 {
            soup.cells.clear();
            let i = ready_cell(&mut soup, 300.0, 300.0);
            for _ in 0..50 {
                soup.divide_ready();
            }
            assert!(soup.cells.iter().all(|c| c.colony.is_none()));
            let _ = i;
            if soup.cells.len() > 1 {
                break;
            }
        }
        assert_eq!(soup.colony_count(), 0, "attachment is for eukaryotes only");
    }

    #[test]
    fn colonies_move_as_one_body_and_stay_in_bounds() {
        let mut soup = bare();
        soup.set_environment(Environment {
            base_temperature: 900.0,
            base_wetness: 0.0,
            ..Environment::default()
        });
        let i = ready_cell(&mut soup, 70.0, 70.0);
        soup.cells[i].nucleus = true;
        soup.cells[i].mitochondria = 1;
        let d = soup.divide(i, true).unwrap();
        let offset =
            |s: &PrimordialSoup| (s.cells[d].x - s.cells[i].x, s.cells[d].y - s.cells[i].y);
        let before = offset(&soup);
        for _ in 0..500 {
            soup.move_cells();
            let (dx, dy) = offset(&soup);
            assert!(
                (dx - before.0).abs() < 1e-6 && (dy - before.1).abs() < 1e-6,
                "members drifted apart"
            );
            for c in &soup.cells {
                assert!(
                    c.x - c.radius() >= -1e-6
                        && c.y - c.radius() >= -1e-6
                        && c.x + c.radius() <= 800.0 + 1e-6
                        && c.y + c.radius() <= 600.0 + 1e-6
                );
            }
        }
        assert!(
            soup.cells[i].x != 70.0 || soup.cells[i].y != 70.0,
            "it did move"
        );
    }

    #[test]
    fn a_colony_left_with_one_member_is_a_cell_again() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].nucleus = true;
        soup.cells[i].mitochondria = 1;
        let d = soup.divide(i, true).unwrap();
        soup.cells.remove(d);
        soup.tidy_colonies();
        assert!(soup.cells[0].colony.is_none() && soup.cells[0].role == Role::Generalist);
        assert_eq!(soup.colony_count(), 0);
    }

    #[test]
    fn feeders_share_food_with_the_colony() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        soup.cells[i].nucleus = true;
        soup.cells[i].mitochondria = 1;
        let d = soup.divide(i, true).unwrap();
        soup.cells[d].role = Role::Feeder;
        soup.cells[d].nucleotides = 10;
        soup.cells[i].nucleotides = 0;
        for _ in 0..6 {
            soup.share_food();
        }
        assert!(soup.cells[i].nucleotides > 0 && soup.cells[d].nucleotides < 10);
        assert_eq!(
            soup.cells[i].nucleotides + soup.cells[d].nucleotides,
            10,
            "food is moved, not made"
        );
    }

    #[test]
    fn a_dead_cell_gives_its_contents_back_to_the_soup() {
        let mut soup = bare();
        let i = ready_cell(&mut soup, 300.0, 300.0);
        let cell = soup.cells.remove(i);
        let (strands, lipids, nucleotides) = (
            cell.strands.len(),
            cell.lipids as usize,
            cell.nucleotides as usize,
        );
        soup.spill(cell);
        assert_eq!(
            (
                soup.strand_count(),
                soup.lipid_count(),
                soup.nucleotide_count()
            ),
            (strands, lipids, nucleotides)
        );
        assert_eq!(soup.cell_stats().deaths, 1);
    }

    #[test]
    fn selection_favours_cells_with_fitter_contents() {
        let divisions = |seq: &'static str| {
            let mut total = 0;
            for seed in 0..20 {
                let mut soup = bare();
                soup.rng = rand::SeedableRng::seed_from_u64(seed);
                let i = ready_cell(&mut soup, 300.0, 300.0);
                soup.cells[i].strands = (0..6).map(|_| strand(seq)).collect();
                for _ in 0..200 {
                    soup.cells.truncate(1);
                    soup.cells[0].strands = (0..6).map(|_| strand(seq)).collect();
                    soup.cells[0].lipids = 10;
                    soup.divide_ready();
                }
                total += soup.cell_stats().divisions;
            }
            total
        };
        assert!(divisions("GAUCAUGC") > divisions("AAAAAAAA"));
    }

    #[test]
    fn a_busy_run_keeps_every_cap_conserves_strands_and_stays_deterministic() {
        let run = || {
            let mut soup = PrimordialSoup::with_seed(600.0, 400.0, 300.0, 7.0, 71);
            soup.add_prebiotic_chemistry();
            soup.seed_cells(8);
            soup.set_environment(Environment::preset(Preset::Both, 300.0, 7.0));
            for _ in 0..4_000 {
                soup.simulate_step();
                assert!(soup.cells.len() <= MAX_CELLS);
                assert!(
                    soup.cells
                        .iter()
                        .all(|c| c.strands.len() <= MAX_CELL_STRANDS
                            && c.mitochondria <= MAX_MITOCHONDRIA)
                );
                assert!(
                    soup.cells
                        .iter()
                        .all(|c| c.lipids <= LIPID_CAP && c.nucleotides <= NUCLEOTIDE_CAP)
                );
                let mut sizes = soup.colony_sizes();
                sizes.retain(|_, n| *n > MAX_COLONY_SIZE);
                assert!(sizes.is_empty());
                assert!(
                    soup.cells
                        .iter()
                        .all(|c| (0.0..=600.0).contains(&c.x) && (0.0..=400.0).contains(&c.y))
                );
                assert!(soup.strand_count() <= MAX_STRANDS);
            }
            let st = soup.cell_stats().clone();
            (
                soup.cells.len(),
                soup.molecules.len(),
                st,
                soup.history.samples().back().map(|s| s.cells),
            )
        };
        let a = run();
        assert_eq!(a, run());
        assert!(a.2.divisions > 0 || a.2.deaths > 0, "something happened");
    }

    #[test]
    fn a_soup_without_cells_has_no_cells() {
        let mut soup = PrimordialSoup::with_seed(300.0, 200.0, 300.0, 7.0, 3);
        soup.add_prebiotic_chemistry();
        for _ in 0..2_000 {
            soup.simulate_step();
        }
        assert!(!soup.cells_enabled() && soup.cells.is_empty() && soup.cell_events().is_empty());
        assert_eq!(soup.mean_cell_fitness(), 0.0);
    }

    #[test]
    fn the_standard_soup_has_the_cell_layer_and_demo_cells_are_capped() {
        let mut soup = PrimordialSoup::standard(800.0, 600.0);
        assert!(soup.cells_enabled());
        soup.seed_cells(100);
        assert_eq!(soup.cell_count(), MAX_CELLS);
        assert!(soup.mean_cell_fitness() > 0.0);
        let _ = total_strands(&soup);
    }

    #[test]
    fn resizing_moves_the_cells_too() {
        let mut soup = bare();
        ready_cell(&mut soup, 400.0, 300.0);
        soup.resize(400.0, 600.0);
        assert_eq!((soup.cells[0].x, soup.cells[0].y), (200.0, 300.0));
    }

    #[test]
    fn a_long_run_reaches_eukaryotes_and_colonies() {
        let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, 4);
        soup.add_small_molecules();
        soup.add_prebiotic_chemistry();
        soup.seed_cells(12);
        soup.set_environment(Environment::preset(Preset::Both, 300.0, 7.0));
        for _ in 0..8_000 {
            soup.simulate_step();
        }
        let st = soup.cell_stats();
        assert!(st.formed > 0 && st.divisions > 0, "{st:?}");
        assert!(st.eukaryotes_formed > 0, "{st:?}");
        assert!(st.colonies_formed > 0, "{st:?}");
        let latest = soup.history.samples().back().expect("sampled");
        assert_eq!(latest.cells, soup.cell_count());
    }
}
