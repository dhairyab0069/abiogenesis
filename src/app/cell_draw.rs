//! How cells look in the soup view: membrane, strands, nucleus, mitochondria and
//! the links that make a colony. Drawing only: what a cell is lives in `soup::cell`.

use super::molecule_draw::base_color;
use crate::soup::{Cell, Role};
use eframe::egui::{self, Color32, Pos2, Stroke, vec2};

const PROKARYOTE: Color32 = Color32::from_rgb(90, 220, 190);
const EUKARYOTE: Color32 = Color32::from_rgb(190, 140, 255);
const NUCLEUS: Color32 = Color32::from_rgb(70, 60, 140);
const MITOCHONDRION: Color32 = Color32::from_rgb(255, 150, 70);

fn role_color(role: Role) -> Option<Color32> {
    match role {
        Role::Generalist => None,
        Role::Germ => Some(Color32::from_rgb(255, 215, 80)),
        Role::Soma => Some(Color32::from_rgb(140, 170, 210)),
        Role::Feeder => Some(Color32::from_rgb(120, 230, 120)),
    }
}

fn colony_color(id: u64) -> Color32 {
    egui::ecolor::Hsva::new(((id as f32) * 0.37).fract(), 0.5, 1.0, 1.0).into()
}

fn centre(origin: Pos2, c: &Cell) -> Pos2 {
    origin + vec2(c.x as f32, c.y as f32)
}

/// Draws all the cells: colony links and halos first, then every cell on top.
pub(super) fn draw_cells(painter: &egui::Painter, origin: Pos2, cells: &[Cell]) {
    for (k, a) in cells.iter().enumerate() {
        let Some(colony) = a.colony else { continue };
        for b in &cells[k + 1..] {
            if b.colony == Some(colony) && (a.x - b.x).hypot(a.y - b.y) <= a.radius() + b.radius() + 2.0 {
                painter.line_segment([centre(origin, a), centre(origin, b)], Stroke::new(7.0, colony_color(colony).gamma_multiply(0.55)));
            }
        }
        painter.circle_stroke(centre(origin, a), a.radius() as f32 + 4.0, Stroke::new(2.5, colony_color(colony).gamma_multiply(0.7)));
    }
    for c in cells {
        draw_cell(painter, centre(origin, c), c);
    }
}

fn draw_cell(painter: &egui::Painter, at: Pos2, cell: &Cell) {
    let r = cell.radius() as f32;
    let tint = if cell.is_eukaryote() { EUKARYOTE } else { PROKARYOTE };
    painter.circle_filled(at, r, tint.gamma_multiply(0.14));
    painter.circle_stroke(at, r, Stroke::new(if cell.is_eukaryote() { 3.0 } else { 2.0 }, tint));
    if cell.is_eukaryote() {
        painter.circle_stroke(at, r - 3.5, Stroke::new(0.8, tint.gamma_multiply(0.4)));
    }

    // the strands: inside the nucleus when there is one, floating free otherwise
    let strand_area = if cell.nucleus { r * 0.4 } else { r * 0.62 };
    let strand_centre = at;
    if cell.nucleus {
        painter.circle_filled(at, r * 0.5, NUCLEUS.gamma_multiply(0.85));
        painter.circle_stroke(at, r * 0.5, Stroke::new(1.5, Color32::from_rgb(150, 140, 230)));
    }
    for (k, strand) in cell.strands.iter().take(6).enumerate() {
        let Some(seq) = strand.strand_sequence() else { continue };
        let angle = k as f32 * 2.399;
        let row = strand_centre + vec2(angle.cos(), angle.sin()) * strand_area * (0.35 + 0.65 * ((k % 3) as f32 / 2.0));
        for (i, base) in seq.chars().take(6).enumerate() {
            painter.circle_filled(row + vec2((i as f32 - 2.5) * 3.2, 0.0), 1.4, base_color(base));
        }
    }

    // mitochondria: orange beans around the nucleus
    for m in 0..cell.mitochondria {
        let angle = 0.9 + m as f32 * 2.1;
        let p = at + vec2(angle.cos(), angle.sin()) * r * 0.74;
        painter.add(egui::epaint::EllipseShape {
            center: p,
            radius: vec2(5.5, 3.2),
            angle: 0.0,
            fill: MITOCHONDRION,
            stroke: Stroke::new(1.0, Color32::from_rgb(120, 60, 20)),
        });
        painter.line_segment([p - vec2(3.0, 0.0), p + vec2(3.0, 0.0)], Stroke::new(0.8, Color32::from_rgb(120, 60, 20)));
    }

    if let Some(color) = role_color(cell.role) {
        painter.circle_filled(at + vec2(0.0, -r - 1.0), 3.5, color);
    }
}

/// The hover text for a cell.
pub(super) fn cell_tooltip(c: &Cell) -> String {
    let kind = if c.is_eukaryote() {
        "Eukaryote"
    } else if c.nucleus || c.mitochondria > 0 {
        "Cell with a partner"
    } else {
        "Cell"
    };
    let mut s = format!("{kind} #{}\nGeneration: {}\nFitness: {:.2}\n", c.id, c.generation, c.fitness());
    s += &format!("Strands: {}   Lipids: {}   Nucleotides: {}\n", c.strands.len(), c.lipids, c.nucleotides);
    if c.nucleus {
        s += "Nucleus: yes (strands last longer and copy better)\n";
    }
    if c.mitochondria > 0 {
        s += &format!("Mitochondria: {} (energy x{:.2})\n", c.mitochondria, c.energy());
    }
    if let Some(colony) = c.colony {
        s += &format!("Colony {colony}, role: {}\n", c.role.label());
    }
    if let Some(parent) = c.parent {
        s += &format!("Divided from cell {parent}\n");
    }
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tooltip_tells_the_cells_story() {
        let mut c = Cell::new(7, 0.0, 0.0, 0);
        c.lipids = 5;
        let plain = cell_tooltip(&c);
        assert!(plain.starts_with("Cell #7") && !plain.contains("Nucleus") && !plain.contains("Colony"));
        c.nucleus = true;
        c.mitochondria = 2;
        c.colony = Some(3);
        c.role = Role::Soma;
        c.parent = Some(2);
        let full = cell_tooltip(&c);
        for needle in ["Eukaryote #7", "Nucleus: yes", "Mitochondria: 2", "Colony 3, role: soma", "Divided from cell 2"] {
            assert!(full.contains(needle), "missing {needle} in {full}");
        }
    }

    #[test]
    fn colony_colours_differ_between_colonies() {
        assert_ne!(colony_color(1), colony_color(2));
        assert!(role_color(Role::Generalist).is_none() && role_color(Role::Germ).is_some());
    }
}
