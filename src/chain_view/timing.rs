//! Sizes and durations of the chain view animations.

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
