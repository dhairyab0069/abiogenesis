//! How a molecule looks in the soup: size, colour and hover text.

use crate::model::Molecule;
use eframe::egui::{self, Color32};

const BASE_MOLECULE_SIZE: f32 = 6.0;

pub(super) fn molecule_size(m: &Molecule) -> f32 {
    if m.is_nucleotide() {
        return 5.0;
    }
    if m.is_lipid() {
        return 8.0;
    }
    BASE_MOLECULE_SIZE + m.energy as f32 * 2.0
}

/// One colour per RNA base.
pub(super) fn base_color(base: char) -> Color32 {
    match base {
        'A' => Color32::from_rgb(255, 120, 120),
        'C' => Color32::from_rgb(110, 170, 255),
        'G' => Color32::from_rgb(110, 230, 140),
        'U' => Color32::from_rgb(255, 220, 100),
        _ => Color32::WHITE,
    }
}

pub(super) const LIPID_COLOR: Color32 = Color32::from_rgb(255, 150, 200);
/// The halo around a strand that carries the catalytic motif.
pub(super) const CATALYST_COLOR: Color32 = Color32::from_rgb(255, 205, 60);

pub(super) fn molecule_color(m: &Molecule) -> Color32 {
    if let Some(base) = m.base() {
        return base_color(base);
    }
    if m.is_lipid() {
        return LIPID_COLOR;
    }
    if m.is_strand() {
        return if m.is_catalyst() { CATALYST_COLOR } else { Color32::from_rgb(150, 170, 255) };
    }
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
    match (m.sequence(), m.strand_sequence()) {
        (Some(seq), _) => s += &format!("Amino Acid Chain\nSequence: {seq}\nLength: {}\n", seq.len()),
        (_, Some(seq)) => {
            s += &format!("RNA Strand\nBases: {seq}\nLength: {}\n", seq.len());
            if let Some(gc) = m.gc_fraction() {
                s += &format!("G and C: {:.0}%\n", gc * 100.0);
            }
            s += if m.is_catalyst() { "Catalyst: yes, it copies itself faster\n" } else { "Catalyst: no\n" };
        }
        _ if m.is_lipid() => s += "Lipid: a water-loving head, an oily tail\n",
        _ if m.is_nucleotide() => s += &format!("Nucleotide: {} (a free RNA building block)\n", m.base().unwrap_or('?')),
        _ => s += &format!("Molecule: {}\n", m.name()),
    }
    s += &format!("Fitness: {:.2}\n", m.fitness());
    s += &format!("Energy: {:.2}\nPosition: ({:.1}, {:.1})\n\nAtomic Composition:\n", m.energy, m.position.x, m.position.y);
    for (el, n) in m.composition() {
        s += &format!("  {el}: {n}\n");
    }
    if let Some(structure) = m.structure() {
        s += &format!("\nStructure:\n  {structure}\n");
    }
    s.trim_end().to_string()
}
