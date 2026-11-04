//! Colours for residues.

/// A hue in 0..1 for a residue, spread so that letters near each other in the
/// alphabet still look different. Unknown letters get 0.
pub fn residue_hue(letter: char) -> f32 {
    let index = crate::model::AMINO_ACIDS.iter().position(|&c| c == letter).unwrap_or(0);
    (index as f32 * 0.381_966).fract()
}
