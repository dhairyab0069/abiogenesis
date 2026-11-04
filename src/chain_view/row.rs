//! Rows: one chain on screen, a line of tiles.

use super::tile::{EditKind, Motion, Origin, Tile};
use super::timing::{GROW_DELAY_PER_TILE, RECOMBINE_SECONDS, SHRINK_SECONDS};
use crate::events::{ChainId, Mutation};

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
    pub(super) fn growing(id: ChainId, sequence: &str) -> Self {
        let tiles = sequence
            .chars()
            .enumerate()
            .map(|(i, letter)| Tile {
                motion: Motion::Growing {
                    elapsed: -(i as f32) * GROW_DELAY_PER_TILE,
                },
                ..Tile::settled(letter)
            })
            .collect();
        Self {
            id,
            tiles,
            origin_left: 0.0,
        }
    }

    /// A recombined row: all tiles present at once so the two colours are visible
    /// together, prefix from parent A and the rest from parent B.
    pub(super) fn recombined(id: ChainId, sequence: &str, prefix_len: usize) -> Self {
        let tiles = sequence
            .chars()
            .enumerate()
            .map(|(i, letter)| Tile {
                origin: if i < prefix_len {
                    Origin::ParentA
                } else {
                    Origin::ParentB
                },
                ..Tile::settled(letter)
            })
            .collect();
        Self {
            id,
            tiles,
            origin_left: RECOMBINE_SECONDS,
        }
    }

    /// The row's chain as it stands: tiles that are shrinking away do not count.
    pub fn sequence(&self) -> String {
        self.tiles
            .iter()
            .filter(|t| !t.is_shrinking())
            .map(|t| t.letter)
            .collect()
    }

    /// How many tiles have started to appear.
    pub fn visible_len(&self) -> usize {
        self.tiles.iter().filter(|t| t.is_visible()).count()
    }

    /// Strength of the parent colouring, 1 fading to 0.
    pub fn origin_strength(&self) -> f32 {
        (self.origin_left / RECOMBINE_SECONDS).clamp(0.0, 1.0)
    }

    pub(super) fn settled(&self) -> bool {
        self.origin_left <= 0.0 && self.tiles.iter().all(|t| t.motion == Motion::Settled)
    }

    fn drop_shrinking(&mut self) {
        self.tiles.retain(|t| !t.is_shrinking());
    }

    /// Applies a logged edit to this row. Returns false if the row cannot take it
    /// (index out of range), so the caller can rebuild.
    pub(super) fn apply(&mut self, mutation: Mutation) -> bool {
        self.drop_shrinking();
        let editing = |kind| Motion::Editing { kind, elapsed: 0.0 };
        match mutation {
            Mutation::Substitution { index, from, to } => {
                let Some(tile) = self.tiles.get_mut(index) else {
                    return false;
                };
                tile.previous = Some(from);
                tile.letter = to;
                tile.motion = editing(EditKind::Substitution);
            }
            Mutation::Insertion { index, residue } => {
                if index > self.tiles.len() {
                    return false;
                }
                let tile = Tile {
                    motion: editing(EditKind::Insertion),
                    ..Tile::settled(residue)
                };
                self.tiles.insert(index, tile);
            }
            Mutation::Deletion { index, .. } => {
                let Some(tile) = self.tiles.get_mut(index) else {
                    return false;
                };
                tile.motion = editing(EditKind::Deletion);
            }
        }
        true
    }

    /// Moves every animation in the row forward by `dt` seconds.
    pub(super) fn advance(&mut self, dt: f32) {
        self.origin_left = (self.origin_left - dt).max(0.0);
        if self.origin_left == 0.0 {
            for t in &mut self.tiles {
                t.origin = Origin::None;
            }
        }
        for t in &mut self.tiles {
            t.advance(dt);
        }
        // A finished deletion takes its tile with it.
        self.tiles.retain(|t| {
            !matches!(t.motion, Motion::Editing { kind: EditKind::Deletion, elapsed } if elapsed >= SHRINK_SECONDS)
        });
    }
}
