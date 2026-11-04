//! The cell inspector: click a cell to see it large, with labels, its real stats
//! and a caption about how lipids make a membrane.

use super::cell_draw::draw_close_up;
use crate::soup::{Cell, PrimordialSoup};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, vec2};

const CAPTION: &str = "Lipids have a water-loving head and two oily tails. In water they turn their heads out and tuck their tails together, \
and two layers back to back close into a sealed bubble: the cell's membrane. What is inside stays inside.";
const SIZE: f32 = 320.0;

/// What a cell is, in a line, for the inspector's header.
pub(super) fn kind_of(c: &Cell) -> &'static str {
    if c.is_eukaryote() {
        "Eukaryote"
    } else if c.nucleus || c.mitochondria > 0 {
        "Cell with a partner"
    } else {
        "Cell"
    }
}

/// The stats shown under the picture, as label and value pairs.
pub(super) fn stats(c: &Cell, now: u64) -> Vec<(&'static str, String)> {
    let mut rows = vec![
        ("Age", format!("{} steps", now.saturating_sub(c.born_step))),
        ("Generation", c.generation.to_string()),
        ("Fitness", format!("{:.2}", c.fitness())),
        ("Lipids", c.lipids.to_string()),
        ("Strands", c.strands.len().to_string()),
        ("Nucleotides", c.nucleotides.to_string()),
        ("Mitochondria", c.mitochondria.to_string()),
        ("Nucleus", if c.nucleus { "yes" } else { "no" }.to_string()),
    ];
    rows.push(("Role", match c.colony {
        Some(colony) => format!("{} (colony {colony})", c.role.label()),
        None => c.role.label().to_string(),
    }));
    rows
}

/// Shows the inspector window for the selected cell. Clears the selection when closed.
pub(super) fn inspector(ctx: &egui::Context, soup: &PrimordialSoup, selected: &mut Option<u64>, time: f32) {
    let Some(id) = *selected else { return };
    let mut open = true;
    egui::Window::new("Cell inspector").open(&mut open).resizable(false).anchor(Align2::LEFT_TOP, vec2(8.0, 8.0)).show(ctx, |ui| {
        let Some(cell) = soup.cells.iter().find(|c| c.id == id) else {
            ui.label(format!("Cell {id} is gone."));
            return;
        };
        ui.strong(format!("{} #{}", kind_of(cell), cell.id));
        let (response, painter) = ui.allocate_painter(vec2(SIZE, SIZE), Sense::hover());
        painter.rect_filled(response.rect, 6.0, Color32::from_rgb(10, 10, 20));
        let centre = response.rect.center();
        let close_up = draw_close_up(&painter, centre, SIZE * 0.36, cell, time);
        let side = response.rect.right() - 6.0;
        let label = |from: Pos2, text: &str, y: f32| {
            let to = Pos2::new(if from.x > centre.x { side } else { response.rect.left() + 6.0 }, y);
            painter.line_segment([from, Pos2::new(to.x + if from.x > centre.x { -4.0 } else { 4.0 }, to.y)], Stroke::new(0.8, Color32::from_white_alpha(120)));
            let align = if from.x > centre.x { Align2::RIGHT_BOTTOM } else { Align2::LEFT_BOTTOM };
            painter.text(to, align, text, FontId::proportional(12.0), Color32::from_white_alpha(220));
        };
        label(close_up.membrane, "membrane: a lipid bilayer", response.rect.top() + 16.0);
        label(close_up.cytoplasm, "cytoplasm", response.rect.bottom() - 10.0);
        if let Some(p) = close_up.nucleus {
            label(p, "nucleus (double membrane, pores)", response.rect.top() + 38.0);
        }
        if let Some(p) = close_up.mitochondrion {
            label(p, "mitochondrion (cristae)", response.rect.bottom() - 30.0);
        }
        if let Some(p) = close_up.strands {
            label(p, "strands", response.rect.bottom() - 50.0);
        }
        egui::Grid::new("inspector_stats").num_columns(2).striped(true).show(ui, |ui| {
            for (k, v) in stats(cell, soup.step) {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            }
        });
        ui.add_space(4.0);
        ui.add(egui::Label::new(egui::RichText::new(CAPTION).weak().small()).wrap());
    });
    if !open {
        *selected = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soup::Role;

    #[test]
    fn the_stats_report_the_cells_real_numbers() {
        let mut c = Cell::new(4, 0.0, 0.0, 100);
        c.lipids = 9;
        c.nucleotides = 3;
        c.role = Role::Germ;
        c.colony = Some(2);
        let rows = stats(&c, 350);
        let get = |k: &str| rows.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone()).unwrap();
        assert_eq!(get("Age"), "250 steps");
        assert_eq!((get("Lipids").as_str(), get("Strands").as_str(), get("Nucleus").as_str()), ("9", "0", "no"));
        assert_eq!(get("Role"), "germ (colony 2)");
        assert_eq!(kind_of(&c), "Cell");
        c.nucleus = true;
        c.mitochondria = 1;
        assert_eq!(kind_of(&c), "Eukaryote");
    }
}
