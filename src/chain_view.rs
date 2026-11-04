//! The view-model behind the chain build-up screen: which rows exist, which tile
//! of each row is animating, and how far along that animation is. No egui in
//! here, so every animation rule can be unit-tested with plain numbers.
//!
//! Design choices:
//! - **Driven by the event log, not by diffing populations.** The view keeps a
//!   cursor into the log and reads only what is new (`EventLog::since`), so it
//!   knows exactly which residue changed and never has to guess from two snapshots.
//! - **The caller passes the live chain ids on every sync.** The log is
//!   append-only and records births, not deaths, so it cannot say a chain is gone
//!   (today only mutation retires a chain, but degradation is planned). Taking
//!   the live set keeps this module independent of `PrimordialSoup` and correct
//!   whatever removes chains in future. Events are applied first, then rows
//!   whose id is not live are dropped, so a row replaced within one batch of
//!   events never flickers out.
//! - **A tile is positioned by its real index from the moment its row exists.**
//!   A growing row holds every tile at once, each hidden until its own start
//!   time, rather than appending tiles one by one. The indices in later events
//!   (`Insertion { index }`, cut points) therefore always line up with the
//!   tiles, even if an edit arrives while the row is still growing.
//! - **A deleted tile stays, shrinking, until its animation ends.** If another
//!   event reaches the row first, pending shrinks are removed on the spot so
//!   that the row's letters match the sequence the event was recorded against.
//! - **After every event the row is checked against the logged sequence** and
//!   rebuilt if they differ, so a missing parent row or a stale row can show a
//!   brief re-grow but never a wrong chain.
//! - **Animations only use elapsed time, and finish by clamping**, so one huge
//!   `advance` settles everything and a paused-then-resumed view cannot stall.

use crate::events::{ChainEvent, ChainId, EventLog, Mutation, UNRECORDED};
use std::collections::HashSet;

/// Most rows kept. Above the soup's own cap (50 chains, plus up to 10 from one
/// step of recombination), so in practice it only guards against a changed soup.
pub const MAX_ROWS: usize = 64;
/// Gap between consecutive tiles of a growing row: one tile at a time.
pub const GROW_DELAY_PER_TILE: f32 = 0.06;
/// How long one tile takes to scale up from nothing.
pub const GROW_SECONDS: f32 = 0.18;
/// How long a substitution flash or an insertion pop lasts.
pub const FLASH_SECONDS: f32 = 0.55;
/// How long a deleted tile takes to shrink away.
pub const SHRINK_SECONDS: f32 = 0.35;
/// How long a recombined row shows which parent each part came from.
pub const RECOMBINE_SECONDS: f32 = 0.9;

/// Which kind of edit a tile is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Substitution,
    Insertion,
    Deletion,
}

impl EditKind {
    fn seconds(self) -> f32 {
        match self {
            Self::Deletion => SHRINK_SECONDS,
            _ => FLASH_SECONDS,
        }
    }
}

/// What a tile is doing right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    /// Growing in. `elapsed` starts negative: the tile is hidden until it reaches 0.
    Growing { elapsed: f32 },
    Settled,
    Editing { kind: EditKind, elapsed: f32 },
}

/// Which parent of a recombination a tile came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    None,
    /// The first parent: the prefix of the child.
    ParentA,
    /// The second parent: the suffix of the child.
    ParentB,
}

/// One residue tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    /// The residue this tile ends up showing.
    pub letter: char,
    /// For a substitution: the letter replaced, shown during the first half of the flash.
    pub previous: Option<char>,
    pub origin: Origin,
    pub motion: Motion,
}

impl Tile {
    fn settled(letter: char) -> Self {
        Self { letter, previous: None, origin: Origin::None, motion: Motion::Settled }
    }

    /// The letter to draw now: a substituted tile shows the old letter first.
    pub fn shown_letter(&self) -> char {
        match (self.previous, self.motion) {
            (Some(old), Motion::Editing { elapsed, .. }) if elapsed < FLASH_SECONDS / 2.0 => old,
            _ => self.letter,
        }
    }

    /// Size factor: 0 is invisible, 1 is full size. Insertions overshoot a little.
    pub fn scale(&self) -> f32 {
        match self.motion {
            Motion::Settled => 1.0,
            Motion::Growing { elapsed } => ease_out((elapsed / GROW_SECONDS).clamp(0.0, 1.0)),
            Motion::Editing { kind: EditKind::Deletion, elapsed } => {
                1.0 - ease_out((elapsed / SHRINK_SECONDS).clamp(0.0, 1.0))
            }
            Motion::Editing { kind: EditKind::Insertion, elapsed } => {
                let t = (elapsed / FLASH_SECONDS).clamp(0.0, 1.0);
                // Rises past full size, then settles back.
                ease_out((t * 2.0).min(1.0)) * (1.0 + 0.25 * (std::f32::consts::PI * t).sin())
            }
            Motion::Editing { .. } => 1.0,
        }
    }

    /// The edit colour to blend in and how strongly (1 at the start, fading to 0).
    pub fn flash(&self) -> Option<(EditKind, f32)> {
        match self.motion {
            Motion::Editing { kind, elapsed } => Some((kind, (1.0 - elapsed / kind.seconds()).clamp(0.0, 1.0))),
            _ => None,
        }
    }

    /// True once the tile has begun to appear (its growth start time has come).
    pub fn is_visible(&self) -> bool {
        !matches!(self.motion, Motion::Growing { elapsed } if elapsed < 0.0)
    }

    fn is_shrinking(&self) -> bool {
        matches!(self.motion, Motion::Editing { kind: EditKind::Deletion, .. })
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// One chain on screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub id: ChainId,
    pub tiles: Vec<Tile>,
    /// Seconds left of the parent-colouring after a recombination; 0 when not showing.
    pub origin_left: f32,
}

impl Row {
    /// A row that grows to `sequence`, one tile at a time.
    fn growing(id: ChainId, sequence: &str) -> Self {
        let tiles = sequence
            .chars()
            .enumerate()
            .map(|(i, letter)| Tile {
                motion: Motion::Growing { elapsed: -(i as f32) * GROW_DELAY_PER_TILE },
                ..Tile::settled(letter)
            })
            .collect();
        Self { id, tiles, origin_left: 0.0 }
    }

    /// A recombined row: all tiles present at once so the two colours are visible
    /// together, prefix from parent A and the rest from parent B.
    fn recombined(id: ChainId, sequence: &str, prefix_len: usize) -> Self {
        let tiles = sequence
            .chars()
            .enumerate()
            .map(|(i, letter)| Tile {
                origin: if i < prefix_len { Origin::ParentA } else { Origin::ParentB },
                ..Tile::settled(letter)
            })
            .collect();
        Self { id, tiles, origin_left: RECOMBINE_SECONDS }
    }

    /// The row's chain as it stands: tiles that are shrinking away do not count.
    pub fn sequence(&self) -> String {
        self.tiles.iter().filter(|t| !t.is_shrinking()).map(|t| t.letter).collect()
    }

    /// How many tiles have started to appear.
    pub fn visible_len(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_visible()).count()
    }

    /// Strength of the parent colouring, 1 fading to 0.
    pub fn origin_strength(&self) -> f32 {
        (self.origin_left / RECOMBINE_SECONDS).clamp(0.0, 1.0)
    }

    fn settled(&self) -> bool {
        self.origin_left <= 0.0 && self.tiles.iter().all(|t| t.motion == Motion::Settled)
    }

    fn drop_shrinking(&mut self) {
        self.tiles.retain(|t| !t.is_shrinking());
    }

    /// Applies a logged edit to this row. Returns false if the row cannot take it
    /// (index out of range), so the caller can rebuild.
    fn apply(&mut self, mutation: Mutation) -> bool {
        self.drop_shrinking();
        let editing = |kind| Motion::Editing { kind, elapsed: 0.0 };
        match mutation {
            Mutation::Substitution { index, from, to } => {
                let Some(tile) = self.tiles.get_mut(index) else { return false };
                tile.previous = Some(from);
                tile.letter = to;
                tile.motion = editing(EditKind::Substitution);
            }
            Mutation::Insertion { index, residue } => {
                if index > self.tiles.len() {
                    return false;
                }
                let tile = Tile { motion: editing(EditKind::Insertion), ..Tile::settled(residue) };
                self.tiles.insert(index, tile);
            }
            Mutation::Deletion { index, .. } => {
                let Some(tile) = self.tiles.get_mut(index) else { return false };
                tile.motion = editing(EditKind::Deletion);
            }
        }
        true
    }

    fn advance(&mut self, dt: f32) {
        self.origin_left = (self.origin_left - dt).max(0.0);
        if self.origin_left == 0.0 {
            for t in &mut self.tiles {
                t.origin = Origin::None;
            }
        }
        for t in &mut self.tiles {
            t.motion = match t.motion {
                Motion::Growing { elapsed } if elapsed + dt >= GROW_SECONDS => Motion::Settled,
                Motion::Growing { elapsed } => Motion::Growing { elapsed: elapsed + dt },
                Motion::Editing { kind, elapsed } if elapsed + dt >= kind.seconds() => {
                    if kind != EditKind::Deletion {
                        t.previous = None;
                    }
                    Motion::Editing { kind, elapsed: kind.seconds() }
                }
                Motion::Editing { kind, elapsed } => Motion::Editing { kind, elapsed: elapsed + dt },
                Motion::Settled => Motion::Settled,
            };
            // A finished flash or pop is just a settled tile again.
            if let Motion::Editing { kind, elapsed } = t.motion
                && kind != EditKind::Deletion
                && elapsed >= kind.seconds()
            {
                t.motion = Motion::Settled;
            }
        }
        // A finished deletion takes its tile with it.
        self.tiles.retain(|t| {
            !matches!(t.motion, Motion::Editing { kind: EditKind::Deletion, elapsed } if elapsed >= SHRINK_SECONDS)
        });
    }
}

/// The rows on screen and the cursor into the event log.
#[derive(Clone, Debug, Default)]
pub struct ChainView {
    rows: Vec<Row>,
    cursor: usize,
}

impl ChainView {
    pub fn new() -> Self {
        Self::default()
    }

    /// The rows, in display order (oldest first; a mutation keeps its parent's place).
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// True while any tile is still moving, so the GUI knows to keep repainting.
    pub fn is_animating(&self) -> bool {
        self.rows.iter().any(|r| !r.settled())
    }

    /// Reads the events recorded since the last sync, then drops rows whose chain
    /// is not in `live`. Chains with no recorded id (`UNRECORDED`) are ignored.
    pub fn sync(&mut self, log: &EventLog, live: impl IntoIterator<Item = ChainId>) {
        for stamped in log.since(self.cursor) {
            self.apply_event(&stamped.event);
        }
        self.cursor = log.len();
        let live: HashSet<ChainId> = live.into_iter().collect();
        self.rows.retain(|r| live.contains(&r.id));
    }

    /// Moves every animation forward by `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        for row in &mut self.rows {
            row.advance(dt.max(0.0));
        }
    }

    fn row_index(&self, id: ChainId) -> Option<usize> {
        if id == UNRECORDED {
            return None;
        }
        self.rows.iter().position(|r| r.id == id)
    }

    fn push(&mut self, row: Row) {
        self.rows.push(row);
        if self.rows.len() > MAX_ROWS {
            self.rows.remove(0);
        }
    }

    fn apply_event(&mut self, event: &ChainEvent) {
        match event {
            ChainEvent::Generated { id, sequence } => self.push(Row::growing(*id, sequence)),
            ChainEvent::Mutated { id, parent, mutation, sequence } => match self.row_index(*parent) {
                // The child takes the parent's place, as it does in the soup.
                Some(i) => {
                    let row = &mut self.rows[i];
                    row.id = *id;
                    if !row.apply(*mutation) || row.sequence() != *sequence {
                        *row = Row::growing(*id, sequence);
                    }
                }
                None => self.push(Row::growing(*id, sequence)),
            },
            ChainEvent::Recombined { id, cut, sequence, .. } => {
                self.push(Row::recombined(*id, sequence, cut.0));
            }
        }
    }
}

/// A hue in 0..1 for a residue, spread so that letters near each other in the
/// alphabet still look different. Unknown letters get 0.
pub fn residue_hue(letter: char) -> f32 {
    let index = crate::model::AMINO_ACIDS.iter().position(|&c| c == letter).unwrap_or(0);
    (index as f32 * 0.381_966).fract()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soup::PrimordialSoup;

    fn generated(log: &mut EventLog, seq: &str) -> ChainId {
        let id = log.new_id();
        log.record(0, ChainEvent::Generated { id, sequence: seq.into() });
        id
    }

    fn mutated(log: &mut EventLog, parent: ChainId, mutation: Mutation, seq: &str) -> ChainId {
        let id = log.new_id();
        log.record(1, ChainEvent::Mutated { id, parent, mutation, sequence: seq.into() });
        id
    }

    fn settle(view: &mut ChainView) {
        view.advance(1000.0);
    }

    fn seqs(view: &ChainView) -> Vec<String> {
        view.rows().iter().map(Row::sequence).collect()
    }

    #[test]
    fn rows_grow_one_tile_at_a_time_to_full_length() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "METHINKS");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        assert_eq!(view.rows()[0].visible_len(), 1, "the first tile starts at once");
        let mut last = view.rows()[0].visible_len();
        for _ in 0..200 {
            view.advance(0.01);
            let now = view.rows()[0].visible_len();
            assert!(now >= last && now - last <= 1, "{last} -> {now}");
            last = now;
        }
        assert_eq!(last, 8);
        assert!(!view.is_animating());
    }

    #[test]
    fn substitution_flashes_exactly_the_recorded_index() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "MET");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        settle(&mut view);
        let b = mutated(&mut log, a, Mutation::Substitution { index: 1, from: 'E', to: 'A' }, "MAT");
        view.sync(&log, [b]);
        let tiles = &view.rows()[0].tiles;
        let flashing: Vec<usize> = (0..3).filter(|&i| tiles[i].flash().is_some()).collect();
        assert_eq!(flashing, vec![1]);
        assert_eq!(tiles[1].flash().map(|f| f.0), Some(EditKind::Substitution));
        assert_eq!(tiles[1].shown_letter(), 'E', "old letter first");
        view.advance(FLASH_SECONDS * 0.75);
        assert_eq!(view.rows()[0].tiles[1].shown_letter(), 'A');
        settle(&mut view);
        assert_eq!(seqs(&view), vec!["MAT"]);
        assert!(view.rows()[0].tiles[1].flash().is_none());
    }

    #[test]
    fn insertion_adds_a_tile_at_the_recorded_index() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "MT");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        settle(&mut view);
        let b = mutated(&mut log, a, Mutation::Insertion { index: 1, residue: 'E' }, "MET");
        view.sync(&log, [b]);
        let tiles = &view.rows()[0].tiles;
        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[1].letter, 'E');
        assert_eq!(tiles[1].flash().map(|f| f.0), Some(EditKind::Insertion));
        assert!(tiles[0].flash().is_none() && tiles[2].flash().is_none());
        settle(&mut view);
        assert_eq!(seqs(&view), vec!["MET"]);
    }

    #[test]
    fn deletion_removes_the_recorded_index_after_shrinking() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "MEET");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        settle(&mut view);
        let b = mutated(&mut log, a, Mutation::Deletion { index: 2, residue: 'E' }, "MET");
        view.sync(&log, [b]);
        assert_eq!(view.rows()[0].tiles.len(), 4, "still there, shrinking");
        assert_eq!(view.rows()[0].sequence(), "MET");
        view.advance(SHRINK_SECONDS / 2.0);
        let s = view.rows()[0].tiles[2].scale();
        assert!(s > 0.0 && s < 1.0);
        view.advance(SHRINK_SECONDS);
        assert_eq!(view.rows()[0].tiles.len(), 3);
        assert_eq!(seqs(&view), vec!["MET"]);
    }

    #[test]
    fn an_edit_during_a_shrink_still_lines_up() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "MEET");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        settle(&mut view);
        let b = mutated(&mut log, a, Mutation::Deletion { index: 0, residue: 'M' }, "EET");
        view.sync(&log, [b]);
        let c = mutated(&mut log, b, Mutation::Substitution { index: 2, from: 'T', to: 'K' }, "EEK");
        view.sync(&log, [c]);
        assert_eq!(view.rows().len(), 1);
        settle(&mut view);
        assert_eq!(seqs(&view), vec!["EEK"]);
    }

    #[test]
    fn recombination_attributes_prefix_and_suffix_by_the_cuts() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "METHINKS");
        let b = generated(&mut log, "WQRPV");
        let c = log.new_id();
        log.record(2, ChainEvent::Recombined { id: c, parents: (a, b), cut: (3, 2), sequence: "METRPV".into() });
        let mut view = ChainView::new();
        view.sync(&log, [a, b, c]);
        let row = &view.rows()[2];
        let origins: Vec<Origin> = row.tiles.iter().map(|t| t.origin).collect();
        let (pa, pb) = (Origin::ParentA, Origin::ParentB);
        assert_eq!(origins, vec![pa, pa, pa, pb, pb, pb]);
        assert_eq!(row.origin_strength(), 1.0);
        view.advance(RECOMBINE_SECONDS + 0.01);
        let row = &view.rows()[2];
        assert!(row.tiles.iter().all(|t| t.origin == Origin::None), "settles back");
        assert_eq!(row.sequence(), "METRPV");
        assert_eq!(view.rows().len(), 3, "a recombination adds a row");
    }

    #[test]
    fn a_mutation_replaces_its_parents_row_in_place() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "AA");
        let b = generated(&mut log, "CC");
        let d = generated(&mut log, "DD");
        let b2 = mutated(&mut log, b, Mutation::Insertion { index: 2, residue: 'K' }, "CCK");
        let mut view = ChainView::new();
        view.sync(&log, [a, b2, d]);
        settle(&mut view);
        assert_eq!(seqs(&view), vec!["AA", "CCK", "DD"]);
        assert_eq!(view.rows()[1].id, b2);
    }

    #[test]
    fn rows_vanish_when_their_chain_is_no_longer_live() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "AA");
        let b = generated(&mut log, "CC");
        let mut view = ChainView::new();
        view.sync(&log, [a, b]);
        view.sync(&log, [b]);
        assert_eq!(seqs(&view), vec!["CC"]);
        view.sync(&log, [UNRECORDED, 99]);
        assert!(view.rows().is_empty());
    }

    #[test]
    fn the_cursor_never_replays_old_events() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "AA");
        let mut view = ChainView::new();
        view.sync(&log, [a]);
        view.sync(&log, [a]);
        assert_eq!(view.rows().len(), 1);
        let b = generated(&mut log, "CC");
        view.sync(&log, [a, b]);
        view.sync(&log, [a, b]);
        assert_eq!(seqs(&view), vec!["AA", "CC"]);
    }

    #[test]
    fn a_mismatched_parent_row_is_rebuilt_not_shown_wrong() {
        let mut log = EventLog::new();
        let b = mutated(&mut log, 77, Mutation::Substitution { index: 0, from: 'A', to: 'C' }, "CAT");
        let mut view = ChainView::new();
        view.sync(&log, [b]);
        settle(&mut view);
        assert_eq!(seqs(&view), vec!["CAT"]);
    }

    #[test]
    fn huge_dt_settles_every_row() {
        let mut log = EventLog::new();
        let a = generated(&mut log, "METHINKS");
        let b = mutated(&mut log, a, Mutation::Insertion { index: 0, residue: 'W' }, "WMETHINKS");
        let c = generated(&mut log, "KK");
        let d = log.new_id();
        log.record(2, ChainEvent::Recombined { id: d, parents: (b, c), cut: (4, 1), sequence: "WMETK".into() });
        let mut view = ChainView::new();
        view.sync(&log, [b, c, d]);
        assert!(view.is_animating());
        view.advance(1.0e9);
        assert!(!view.is_animating());
        assert!(view.rows().iter().all(|r| r.tiles.iter().all(|t| t.motion == Motion::Settled && t.scale() == 1.0)));
    }

    #[test]
    fn rows_are_capped() {
        let mut log = EventLog::new();
        let ids: Vec<_> = (0..MAX_ROWS + 5).map(|_| generated(&mut log, "A")).collect();
        let mut view = ChainView::new();
        view.sync(&log, ids.clone());
        assert_eq!(view.rows().len(), MAX_ROWS);
        assert_eq!(view.rows()[0].id, ids[5], "the oldest rows go first");
    }

    #[test]
    fn follows_a_real_soup_for_two_thousand_steps() {
        for seed in [1, 2, 3] {
            let mut soup = PrimordialSoup::with_seed(800.0, 600.0, 300.0, 7.0, seed);
            let mut view = ChainView::new();
            for _ in 0..2000 {
                soup.simulate_step();
                let live = soup.molecules.iter().filter(|m| m.is_amino_acid()).map(|m| m.id);
                view.sync(&soup.events, live);
                view.advance(1.0 / 20.0);
            }
            assert!(view.rows().len() <= MAX_ROWS);
            let mut shown = seqs(&view);
            let mut actual: Vec<String> =
                soup.molecules.iter().filter_map(|m| m.sequence()).map(str::to_string).collect();
            shown.sort();
            actual.sort();
            assert_eq!(shown, actual, "seed {seed}");
        }
    }
}
