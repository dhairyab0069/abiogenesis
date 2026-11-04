//! Tiles: one residue each, with the animation state of its growth or edit.

use super::timing::{FLASH_SECONDS, GROW_SECONDS, SHRINK_SECONDS};

/// Which kind of edit a tile is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Substitution,
    Insertion,
    Deletion,
}

impl EditKind {
    pub(super) fn seconds(self) -> f32 {
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
    Growing {
        elapsed: f32,
    },
    Settled,
    Editing {
        kind: EditKind,
        elapsed: f32,
    },
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
    pub(super) fn settled(letter: char) -> Self {
        Self {
            letter,
            previous: None,
            origin: Origin::None,
            motion: Motion::Settled,
        }
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
            Motion::Editing {
                kind: EditKind::Deletion,
                elapsed,
            } => 1.0 - ease_out((elapsed / SHRINK_SECONDS).clamp(0.0, 1.0)),
            Motion::Editing {
                kind: EditKind::Insertion,
                elapsed,
            } => {
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
            Motion::Editing { kind, elapsed } => {
                Some((kind, (1.0 - elapsed / kind.seconds()).clamp(0.0, 1.0)))
            }
            _ => None,
        }
    }

    /// True once the tile has begun to appear (its growth start time has come).
    pub fn is_visible(&self) -> bool {
        !matches!(self.motion, Motion::Growing { elapsed } if elapsed < 0.0)
    }

    pub(super) fn is_shrinking(&self) -> bool {
        matches!(
            self.motion,
            Motion::Editing {
                kind: EditKind::Deletion,
                ..
            }
        )
    }

    /// Moves this tile's animation forward by `dt` seconds, clamping at the end.
    pub(super) fn advance(&mut self, dt: f32) {
        self.motion = match self.motion {
            Motion::Growing { elapsed } if elapsed + dt >= GROW_SECONDS => Motion::Settled,
            Motion::Growing { elapsed } => Motion::Growing {
                elapsed: elapsed + dt,
            },
            Motion::Editing { kind, elapsed } if elapsed + dt >= kind.seconds() => {
                if kind != EditKind::Deletion {
                    self.previous = None;
                }
                Motion::Editing {
                    kind,
                    elapsed: kind.seconds(),
                }
            }
            Motion::Editing { kind, elapsed } => Motion::Editing {
                kind,
                elapsed: elapsed + dt,
            },
            Motion::Settled => Motion::Settled,
        };
        // A finished flash or pop is just a settled tile again.
        if let Motion::Editing { kind, elapsed } = self.motion
            && kind != EditKind::Deletion
            && elapsed >= kind.seconds()
        {
            self.motion = Motion::Settled;
        }
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}
