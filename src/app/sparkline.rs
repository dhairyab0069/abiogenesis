//! A tiny line chart drawn with the painter: a label, the newest value and the
//! shape of the last few minutes.

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Stroke, vec2};

const HEIGHT: f32 = 38.0;

/// Draws `values` (oldest first) as a line scaled to its own minimum and maximum.
pub(super) fn sparkline(
    ui: &mut egui::Ui,
    label: &str,
    values: &[f64],
    color: Color32,
    decimals: usize,
) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 3.0, ui.visuals().extreme_bg_color);
    let text = ui.visuals().weak_text_color();
    painter.text(
        rect.left_top() + vec2(5.0, 3.0),
        Align2::LEFT_TOP,
        label,
        FontId::proportional(10.5),
        text,
    );
    let latest = values.last().copied();
    if let Some(v) = latest {
        painter.text(
            rect.right_top() + vec2(-5.0, 3.0),
            Align2::RIGHT_TOP,
            format!("{v:.decimals$}"),
            FontId::monospace(11.0),
            color,
        );
    }
    let points = chart_points(
        values,
        rect.shrink2(vec2(5.0, 0.0))
            .with_min_y(rect.top() + 16.0)
            .with_max_y(rect.bottom() - 4.0),
    );
    if points.len() >= 2 {
        painter.add(egui::Shape::line(points, Stroke::new(1.6, color)));
    }
    if let Some((lo, hi)) = bounds(values) {
        response.on_hover_text(format!(
            "{label}\nmin {lo:.decimals$}, max {hi:.decimals$}, {} samples",
            values.len()
        ));
    }
}

fn bounds(values: &[f64]) -> Option<(f64, f64)> {
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (lo.is_finite() && hi.is_finite()).then_some((lo, hi))
}

/// Screen points for `values` inside `area`: oldest at the left, the minimum at the
/// bottom and the maximum at the top. A flat series sits in the middle.
pub(super) fn chart_points(values: &[f64], area: egui::Rect) -> Vec<Pos2> {
    let Some((lo, hi)) = bounds(values) else {
        return Vec::new();
    };
    let span = hi - lo;
    let last = (values.len().max(2) - 1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let t = if span > 0.0 {
                ((v - lo) / span) as f32
            } else {
                0.5
            };
            Pos2::new(
                area.left() + area.width() * i as f32 / last,
                area.bottom() - area.height() * t,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_span_the_area_with_the_minimum_at_the_bottom() {
        let area = egui::Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 20.0));
        let pts = chart_points(&[1.0, 3.0, 2.0], area);
        assert_eq!(pts.len(), 3);
        assert_eq!((pts[0].x, pts[2].x), (0.0, 100.0));
        assert_eq!((pts[0].y, pts[1].y), (20.0, 0.0));
        assert!((pts[2].y - 10.0).abs() < 1e-4);
    }

    #[test]
    fn flat_and_empty_series_are_safe() {
        let area = egui::Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 20.0));
        assert!(chart_points(&[], area).is_empty());
        assert!(
            chart_points(&[5.0, 5.0, 5.0], area)
                .iter()
                .all(|p| p.y == 10.0)
        );
        assert_eq!(chart_points(&[7.0], area).len(), 1);
    }
}
