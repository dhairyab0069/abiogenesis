//! How a molecule looks in the soup: size, colour and hover text.

use crate::model::Molecule;
use eframe::egui::{self, Color32};

const BASE_MOLECULE_SIZE: f32 = 6.0;

pub(super) fn molecule_size(m: &Molecule) -> f32 {
    BASE_MOLECULE_SIZE + m.energy as f32 * 2.0
}

pub(super) fn molecule_color(m: &Molecule) -> Color32 {
    match (m.sequence(), m.name().as_str()) {
        // Chains are coloured by length.
        (Some(seq), _) => egui::ecolor::Hsva::new(((seq.len() * 20) % 360) as f32 / 360.0, 0.8, 1.0, 1.0).into(),
        (None, "H2O") => Color32::from_rgb(0, 255, 255),
        (None, "CH4") => Color32::from_rgb(144, 238, 144),
        (None, "CH3OH") => Color32::from_rgb(128, 0, 128),
        _ => Color32::WHITE,
    }
}

pub(super) fn tooltip_text(m: &Molecule) -> String {
    let mut s = String::new();
    match m.sequence() {
        Some(seq) => s += &format!("Amino Acid Chain\nSequence: {seq}\nLength: {}\n", seq.len()),
        None => s += &format!("Molecule: {}\n", m.name()),
    }
    s += &format!("Energy: {:.2}\nPosition: ({:.1}, {:.1})\n\nAtomic Composition:\n", m.energy, m.position.x, m.position.y);
    for (el, n) in m.composition() {
        s += &format!("  {el}: {n}\n");
    }
    if let Some(structure) = m.structure() {
        s += &format!("\nStructure:\n  {structure}\n");
    }
    s.trim_end().to_string()
}
