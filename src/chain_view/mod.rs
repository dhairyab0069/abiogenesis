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


mod apply;
mod hue;
mod row;
mod tile;
mod timing;
#[cfg(test)]
mod tests;

pub use hue::residue_hue;
pub use row::Row;
pub use tile::{EditKind, Motion, Origin, Tile};
pub use timing::{FLASH_SECONDS, GROW_DELAY_PER_TILE, GROW_SECONDS, MAX_ROWS, RECOMBINE_SECONDS, SHRINK_SECONDS};

use crate::events::{ChainId, EventLog};
use std::collections::HashSet;

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

}
