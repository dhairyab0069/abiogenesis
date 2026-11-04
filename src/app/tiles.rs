//! Drawing for the chains screen: the legend and the rows of letter tiles.

use crate::chain_view::{EditKind, Origin, Row, Tile, residue_hue};
use eframe::egui::{self, Color32, Pos2, Sense, Vec2};

const TILE_SIZE: f32 = 24.0;
const TILE_GAP: f32 = 3.0;
const ROW_HEIGHT: f32 = TILE_SIZE + 6.0;
const SUBSTITUTION_COLOR: Color32 = Color32::from_rgb(0xff, 0xb3, 0x00);
const INSERTION_COLOR: Color32 = Color32::from_rgb(0x69, 0xf0, 0xae);
const DELETION_COLOR: Color32 = Color32::from_rgb(0xff, 0x52, 0x52);
const PARENT_A_COLOR: Color32 = Color32::from_rgb(0x40, 0xc4, 0xff);
const PARENT_B_COLOR: Color32 = Color32::from_rgb(0xea, 0x80, 0xfc);

/// The colour key for edits and recombination parents.
pub(super) fn legend_ui(ui: &mut egui::Ui) {
    legend_item(ui, SUBSTITUTION_COLOR, "substitution");
    legend_item(ui, INSERTION_COLOR, "insertion");
    legend_item(ui, DELETION_COLOR, "deletion");
    legend_item(ui, PARENT_A_COLOR, "recombination: from parent A (start)");
    legend_item(ui, PARENT_B_COLOR, "from parent B (end)");
}

fn legend_item(ui: &mut egui::Ui, color: Color32, label: &str) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, color);
    ui.label(label);
    ui.add_space(8.0);
}

/// Draws one chain as a row of letter tiles. Each tile advances the cursor by its
/// own scale so rows reflow smoothly as tiles grow in and shrink away.
pub(super) fn draw_row(ui: &mut egui::Ui, row: &Row) {
    let (response, painter) =
        ui.allocate_painter(Vec2::new(ui.available_width(), ROW_HEIGHT), Sense::hover());
    let mut x = response.rect.left();
    let cy = response.rect.center().y;
    for tile in &row.tiles {
        let scale = tile.scale();
        let size = TILE_SIZE * scale;
        if size > 0.5 {
            let rect = egui::Rect::from_center_size(
                Pos2::new(x + TILE_SIZE * scale.min(1.0) / 2.0, cy),
                Vec2::splat(size),
            );
            let color = tile_color(tile, row.origin_strength());
            painter.rect_filled(rect, 4.0, color);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                tile.shown_letter(),
                egui::FontId::monospace(16.0 * scale.min(1.0)),
                Color32::BLACK,
            );
        }
        x += (TILE_SIZE + TILE_GAP) * scale.min(1.0);
    }
}

fn tile_color(tile: &Tile, origin_strength: f32) -> Color32 {
    let base: Color32 = egui::ecolor::Hsva::new(residue_hue(tile.letter), 0.45, 0.9, 1.0).into();
    let base = match tile.origin {
        Origin::None => base,
        Origin::ParentA => base.lerp_to_gamma(PARENT_A_COLOR, origin_strength),
        Origin::ParentB => base.lerp_to_gamma(PARENT_B_COLOR, origin_strength),
    };
    match tile.flash() {
        Some((kind, strength)) => {
            let flash = match kind {
                EditKind::Substitution => SUBSTITUTION_COLOR,
                EditKind::Insertion => INSERTION_COLOR,
                EditKind::Deletion => DELETION_COLOR,
            };
            base.lerp_to_gamma(flash, strength)
        }
        None => base,
    }
}
