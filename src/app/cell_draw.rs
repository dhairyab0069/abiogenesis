//! How cells look in the soup view and in the inspector: a membrane made of lipid
//! glyphs (or a smooth double line when small), cytoplasm, strands, a nucleus with a
//! double membrane and pores, mitochondria with cristae, and the animations of
//! forming, dividing and swallowing. Drawing only: what a cell is lives in
//! `soup::cell`, and the geometry and timelines are in `crate::cellviz`.

use super::molecule_draw::base_color;
use crate::cellviz::{self as viz, Detail, Leaflet, Pt};
use crate::soup::{Cell, PrimordialSoup, Role};
use eframe::egui::{self, Color32, Pos2, Stroke, epaint::EllipseShape};
use std::cell::Cell as Counter;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const PROKARYOTE: Color32 = Color32::from_rgb(70, 190, 170);
const EUKARYOTE: Color32 = Color32::from_rgb(160, 120, 230);
const NUCLEUS: Color32 = Color32::from_rgb(70, 60, 140);
const NUCLEUS_EDGE: Color32 = Color32::from_rgb(170, 160, 240);
const MITOCHONDRION: Color32 = Color32::from_rgb(255, 150, 70);
const MITO_EDGE: Color32 = Color32::from_rgb(120, 60, 20);
const OUTER_HEAD: Color32 = Color32::from_rgb(255, 160, 185);
const INNER_HEAD: Color32 = Color32::from_rgb(255, 200, 140);
const TAIL: Color32 = Color32::from_rgb(255, 235, 205);
/// Lipid glyphs drawn per frame at most, over all cells.
const GLYPH_BUDGET: usize = 1_800;

fn role_tint(role: Role) -> Option<Color32> {
    match role {
        Role::Generalist => None,
        Role::Germ => Some(Color32::from_rgb(255, 215, 80)),
        Role::Soma => Some(Color32::from_rgb(140, 170, 210)),
        Role::Feeder => Some(Color32::from_rgb(120, 230, 120)),
    }
}

fn at(origin: Pos2, p: Pt) -> Pos2 {
    Pos2::new(origin.x + p.x, origin.y + p.y)
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

fn rot(p: Pt, angle: f32) -> Pt {
    let (s, c) = angle.sin_cos();
    Pt::new(p.x * c - p.y * s, p.x * s + p.y * c)
}

/// How one cell is drawn this frame.
#[derive(Clone, Default)]
struct Look {
    /// Where it is drawn (soup position, possibly animated).
    centre: Pt,
    /// Neighbours (centre, radius) whose insides hide this cell's membrane: colony
    /// mates, and the other half of a dividing cell.
    neighbours: Vec<(Pt, f32)>,
    /// Formation progress (0..1) and where its lipids and strands started, relative to the centre.
    forming: Option<(f32, Vec<Pt>, Vec<Pt>)>,
    /// Mitochondria and nucleus held back while a swallowing plays.
    hide_mitos: u32,
    nucleus_alpha: f32,
}

/// Draws all the cells of the soup.
pub(super) fn draw_cells(painter: &egui::Painter, origin: Pos2, soup: &PrimordialSoup, time: f32) {
    let budget = Counter::new(GLYPH_BUDGET);
    let looks = looks(soup);
    let radius = |c: &Cell| c.radius() as f32;
    for c in &soup.cells {
        let look = looks.get(&c.id).cloned().unwrap_or_default();
        draw_body(painter, origin, c, radius(c), &look, time, &budget);
    }
    // shared walls and junctions between colony mates
    for (k, a) in soup.cells.iter().enumerate() {
        let Some(colony) = a.colony else { continue };
        for b in &soup.cells[k + 1..] {
            if b.colony != Some(colony) {
                continue;
            }
            let (ca, cb) = (looks[&a.id].centre, looks[&b.id].centre);
            if let Some((p, q)) = viz::circle_intersection(ca, radius(a), cb, radius(b)) {
                painter.line_segment(
                    [at(origin, p), at(origin, q)],
                    Stroke::new(2.0, OUTER_HEAD.gamma_multiply(0.8)),
                );
                for f in [0.25, 0.5, 0.75] {
                    let m = at(origin, p.lerp(q, f));
                    painter.circle_filled(m, 1.8, Color32::WHITE.gamma_multiply(0.85));
                    painter.circle_stroke(m, 2.8, Stroke::new(0.8, TAIL.gamma_multiply(0.7)));
                }
            }
        }
    }
    draw_swallowings(painter, origin, soup, &looks);
}

/// The look of every cell: its position (animated while dividing), who hides its
/// membrane, and any formation or swallowing in progress.
fn looks(soup: &PrimordialSoup) -> HashMap<u64, Look> {
    let step = soup.step as f32;
    let mut looks: HashMap<u64, Look> = soup
        .cells
        .iter()
        .map(|c| {
            (
                c.id,
                Look {
                    centre: Pt::new(c.x as f32, c.y as f32),
                    nucleus_alpha: 1.0,
                    ..Look::default()
                },
            )
        })
        .collect();
    let pos = |c: &Cell| Pt::new(c.x as f32, c.y as f32);
    let by_id: HashMap<u64, &Cell> = soup.cells.iter().map(|c| (c.id, c)).collect();

    // division: both halves start as one elongated cell and part
    for d in &soup.cells {
        let Some(m) = d.parent.and_then(|p| by_id.get(&p)) else {
            continue;
        };
        let Some(t) = viz::pinch_progress(step - d.born_step as f32) else {
            continue;
        };
        let (cm, cd) = viz::pinch_centres(pos(m), pos(d), t);
        looks.get_mut(&m.id).expect("mother").centre = cm;
        looks.get_mut(&d.id).expect("daughter").centre = cd;
        looks
            .get_mut(&m.id)
            .expect("mother")
            .neighbours
            .push((cd, d.radius() as f32));
        looks
            .get_mut(&d.id)
            .expect("daughter")
            .neighbours
            .push((cm, m.radius() as f32));
    }
    // colony mates press together and share walls
    for (k, a) in soup.cells.iter().enumerate() {
        let Some(colony) = a.colony else { continue };
        for b in &soup.cells[k + 1..] {
            if b.colony == Some(colony) {
                let (ca, cb) = (looks[&a.id].centre, looks[&b.id].centre);
                looks
                    .get_mut(&a.id)
                    .expect("a")
                    .neighbours
                    .push((cb, b.radius() as f32));
                looks
                    .get_mut(&b.id)
                    .expect("b")
                    .neighbours
                    .push((ca, a.radius() as f32));
            }
        }
    }
    // formation
    for rec in soup.formation_records() {
        let Some(c) = by_id.get(&rec.cell_id) else {
            continue;
        };
        let Some(t) = viz::formation_progress(step - rec.step as f32) else {
            continue;
        };
        let rel =
            |p: &(f64, f64)| Pt::new((p.0 - rec.centre.0) as f32, (p.1 - rec.centre.1) as f32);
        let look = looks.get_mut(&c.id).expect("cell");
        look.forming = Some((
            t,
            rec.lipids.iter().map(rel).collect(),
            rec.strands.iter().map(rel).collect(),
        ));
    }
    // swallowing: hold back what the guest turns into
    for rec in soup.engulf_records() {
        let Some(c) = by_id.get(&rec.host_id) else {
            continue;
        };
        let age = step - rec.step as f32;
        if (0.0..viz::ENGULF_STEPS).contains(&age) {
            let look = looks.get_mut(&c.id).expect("host");
            look.hide_mitos = 1;
            if rec.new_nucleus {
                look.nucleus_alpha = viz::ease((age / viz::ENGULF_STEPS - 0.6) / 0.4);
            }
        }
    }
    looks
}

/// The cup round a guest, and the guest shrinking into a mitochondrion.
fn draw_swallowings(
    painter: &egui::Painter,
    origin: Pos2,
    soup: &PrimordialSoup,
    looks: &HashMap<u64, Look>,
) {
    let step = soup.step as f32;
    for rec in soup.engulf_records() {
        let Some(host) = soup.cells.iter().find(|c| c.id == rec.host_id) else {
            continue;
        };
        let r = host.radius() as f32;
        let (slot, len) = viz::mito_slot(host.mitochondria.saturating_sub(1), r);
        let guest = Pt::new(rec.guest_offset.0 as f32, rec.guest_offset.1 as f32);
        let Some(cup) = viz::cup(
            step - rec.step as f32,
            guest,
            rec.guest_radius as f32,
            slot,
            len * 0.6,
        ) else {
            continue;
        };
        let host_at = looks
            .get(&host.id)
            .map_or(Pt::new(host.x as f32, host.y as f32), |l| l.centre);
        let centre = at(origin, host_at + cup.guest);
        let tint = mix(PROKARYOTE, MITOCHONDRION, cup.shrunk);
        let gr = cup.guest_radius;
        painter.circle_filled(centre, gr, tint.gamma_multiply(0.35));
        painter.circle_stroke(centre, gr, Stroke::new(1.6, tint));
        // the host's arm: an arc round the guest, opening away from the host until it closes
        let toward_host = (cup.guest * -1.0).y.atan2((cup.guest * -1.0).x);
        let points: Vec<Pos2> = (0..=24)
            .map(|k| {
                let a = toward_host - cup.span / 2.0 + cup.span * k as f32 / 24.0;
                centre + egui::vec2(a.cos(), a.sin()) * (gr + 3.0)
            })
            .collect();
        let arm = Stroke::new(2.8, OUTER_HEAD.gamma_multiply(1.0 - 0.6 * cup.shrunk));
        painter.add(egui::Shape::line(points, arm));
    }
}

fn draw_body(
    painter: &egui::Painter,
    origin: Pos2,
    cell: &Cell,
    r: f32,
    look: &Look,
    time: f32,
    budget: &Counter<usize>,
) {
    let c = look.centre;
    let centre = at(origin, c);
    let spec = viz::bilayer_spec(r);
    let forming = look.forming.as_ref().map(|(t, _, _)| *t);
    let fill = forming.map_or(1.0, viz::fill_alpha);
    let tint = {
        let base = if cell.is_eukaryote() {
            EUKARYOTE
        } else {
            PROKARYOTE
        };
        role_tint(cell.role).map_or(base, |t| mix(base, t, 0.35))
    };

    // cytoplasm: a soft gradient, a few drifting specks
    let inner = spec.inner_edge.max(r * 0.3);
    painter.circle_filled(centre, r, tint.gamma_multiply(0.07 * fill));
    for k in 0..6 {
        painter.circle_filled(
            centre,
            inner * (1.0 - 0.13 * k as f32),
            tint.gamma_multiply(0.055 * fill),
        );
    }
    for k in 0..9u64 {
        let a = viz::hash01(cell.id, k) * TAU + time * 0.05 * (k % 3) as f32;
        let d = viz::hash01(cell.id, k + 40).sqrt() * inner * 0.8;
        let p = centre + egui::vec2(a.cos(), a.sin()) * d;
        painter.circle_filled(
            p,
            (r * 0.02).max(0.6),
            Color32::WHITE.gamma_multiply(0.12 * fill),
        );
    }

    draw_strands(painter, origin, cell, r, look, inner, fill);
    if cell.nucleus && look.nucleus_alpha > 0.01 {
        draw_nucleus(painter, centre, r, look.nucleus_alpha);
    }
    for m in 0..cell.mitochondria.saturating_sub(look.hide_mitos) {
        draw_mitochondrion(painter, centre, r, m, 1.0);
    }

    // the membrane
    let skip = |p: Pt| {
        look.neighbours
            .iter()
            .any(|(nc, nr)| (p + c).dist(*nc) < nr - spec.head_radius)
    };
    let want = if forming.is_some() {
        Detail::Glyphs
    } else {
        viz::detail(r, budget.get())
    };
    if want == Detail::Glyphs && viz::glyph_total(r) <= budget.get() {
        budget.set(budget.get() - viz::glyph_total(r));
        draw_glyph_membrane(painter, centre, cell.id, r, look, &skip, time);
    } else {
        draw_smooth_membrane(painter, c, origin, r, tint, &skip, cell.is_eukaryote());
    }
    if let Some(color) = role_tint(cell.role) {
        painter.circle_filled(
            centre + egui::vec2(0.0, -r - 3.0),
            (r * 0.09).max(2.5),
            color,
        );
    }
}

fn draw_smooth_membrane(
    painter: &egui::Painter,
    c: Pt,
    origin: Pos2,
    r: f32,
    tint: Color32,
    skip: &dyn Fn(Pt) -> bool,
    euk: bool,
) {
    let spec = viz::bilayer_spec(r);
    let width = (spec.head_radius * 1.3).max(1.2);
    for (ring, color) in [
        (spec.mid + spec.tail_len + spec.head_radius, OUTER_HEAD),
        (spec.mid - spec.tail_len - spec.head_radius, INNER_HEAD),
    ] {
        for run in viz::membrane_runs(Pt::new(0.0, 0.0), ring, 64, skip) {
            let pts: Vec<Pos2> = run.into_iter().map(|p| at(origin, p + c)).collect();
            if pts.len() >= 2 {
                painter.add(egui::Shape::line(
                    pts,
                    Stroke::new(width, mix(color, tint, if euk { 0.3 } else { 0.1 })),
                ));
            }
        }
    }
}

fn draw_glyph_membrane(
    painter: &egui::Painter,
    centre: Pos2,
    seed: u64,
    r: f32,
    look: &Look,
    skip: &dyn Fn(Pt) -> bool,
    time: f32,
) {
    let glyphs = viz::bilayer(r);
    let outer_n = glyphs
        .iter()
        .filter(|g| g.leaflet == Leaflet::Outer)
        .count();
    let tail_width = (viz::bilayer_spec(r).head_radius * 0.35).max(0.7);
    for (i, g) in glyphs.iter().enumerate() {
        let (mut head, mut tails, mut alpha) = (g.head, g.tails, 1.0);
        if let Some((t, lipid_starts, _)) = &look.forming {
            if g.leaflet == Leaflet::Outer {
                // the glyph travels from where its lipid was, turning its head outward
                let start = if lipid_starts.is_empty() {
                    g.head * 2.0
                } else {
                    lipid_starts[i % lipid_starts.len()]
                        + Pt::polar(i as f32 * 2.4, 3.0 + (i % 5) as f32)
                };
                let pose = viz::outer_pose(i, outer_n, start, seed, g, *t);
                head = pose.pos;
                let (shift, turned) = (pose.pos - g.head * viz::curl(*t), pose.angle - g.angle);
                tails = g.tails.map(|(a, b)| {
                    (
                        rot(a, turned) * viz::curl(*t) + shift,
                        rot(b, turned) * viz::curl(*t) + shift,
                    )
                });
            } else {
                alpha = viz::inner_alpha(*t);
                if alpha <= 0.0 {
                    continue;
                }
            }
        } else if skip(g.head) {
            continue;
        }
        let (radial, side) = viz::wobble(i, time);
        let dir = Pt::polar(g.angle, 1.0);
        let jiggle = dir * (radial * 0.22 * g.head_radius)
            + Pt::new(-dir.y, dir.x) * (side * 0.22 * g.head_radius);
        let shimmer = alpha * (0.85 + 0.15 * (time * 1.7 + i as f32).sin());
        let (head_color, tail_color) = match g.leaflet {
            Leaflet::Outer => (OUTER_HEAD, TAIL),
            Leaflet::Inner => (INNER_HEAD, TAIL),
        };
        for (a, b) in tails {
            painter.line_segment(
                [at_pos(centre, a + jiggle), at_pos(centre, b + jiggle)],
                Stroke::new(tail_width, tail_color.gamma_multiply(0.7 * shimmer)),
            );
        }
        painter.circle_filled(
            at_pos(centre, head + jiggle),
            g.head_radius,
            head_color.gamma_multiply(shimmer),
        );
    }
}

fn at_pos(centre: Pos2, p: Pt) -> Pos2 {
    Pos2::new(centre.x + p.x, centre.y + p.y)
}

/// Bead rows: inside the nucleus when there is one, floating in the cytoplasm otherwise.
fn draw_strands(
    painter: &egui::Painter,
    origin: Pos2,
    cell: &Cell,
    r: f32,
    look: &Look,
    inner: f32,
    fill: f32,
) {
    let area = if cell.nucleus {
        viz::nucleus_radius(r) * 0.55
    } else {
        inner * 0.6
    };
    let bead = (r * 0.035).max(1.0);
    let beads = if r > 80.0 { 10 } else { 6 };
    let centre = at(origin, look.centre);
    for (k, strand) in cell.strands.iter().take(6).enumerate() {
        let Some(seq) = strand.strand_sequence() else {
            continue;
        };
        let angle = k as f32 * 2.399;
        let slot = Pt::polar(angle, area * (0.3 + 0.7 * ((k % 3) as f32 / 2.0)));
        let from = look
            .forming
            .as_ref()
            .and_then(|(_, _, s)| s.get(k).copied());
        let pos = from.map_or(slot, |s| s.lerp(slot, fill));
        let dir = Pt::polar(angle * 0.7 + 0.5, 1.0);
        let n = seq.chars().count().min(beads) as f32;
        let mut last: Option<Pos2> = None;
        for (i, base) in seq.chars().take(beads).enumerate() {
            let p = centre
                + egui::vec2(pos.x, pos.y)
                + egui::vec2(dir.x, dir.y) * ((i as f32 - (n - 1.0) / 2.0) * bead * 2.5);
            if let Some(l) = last {
                painter.line_segment(
                    [l, p],
                    Stroke::new(bead * 0.4, Color32::WHITE.gamma_multiply(0.25)),
                );
            }
            painter.circle_filled(p, bead, base_color(base));
            last = Some(p);
        }
    }
}

/// A nucleus: a double membrane with pores.
fn draw_nucleus(painter: &egui::Painter, centre: Pos2, r: f32, alpha: f32) {
    let nr = viz::nucleus_radius(r);
    let gap = (r * 0.03).max(1.2);
    painter.circle_filled(centre, nr, NUCLEUS.gamma_multiply(0.85 * alpha));
    painter.circle_stroke(
        centre,
        nr,
        Stroke::new((r * 0.025).max(1.2), NUCLEUS_EDGE.gamma_multiply(alpha)),
    );
    painter.circle_stroke(
        centre,
        nr - gap,
        Stroke::new(
            (r * 0.015).max(0.8),
            NUCLEUS_EDGE.gamma_multiply(0.6 * alpha),
        ),
    );
    for k in 0..10 {
        let a = k as f32 / 10.0 * TAU + 0.3;
        let p = centre + egui::vec2(a.cos(), a.sin()) * (nr - gap / 2.0);
        painter.circle_filled(p, gap * 0.55, NUCLEUS.gamma_multiply(alpha));
        painter.circle_stroke(
            p,
            gap * 0.55,
            Stroke::new(0.6, NUCLEUS_EDGE.gamma_multiply(0.8 * alpha)),
        );
    }
}

/// A mitochondrion: a bean with a double membrane and inner folds (cristae).
fn draw_mitochondrion(painter: &egui::Painter, centre: Pos2, r: f32, index: u32, alpha: f32) {
    let (slot, len) = viz::mito_slot(index, r);
    let angle = slot.y.atan2(slot.x) + PI / 2.0;
    let c = centre + egui::vec2(slot.x, slot.y);
    let w = len * 0.55;
    painter.add(EllipseShape {
        center: c,
        radius: egui::vec2(len, w),
        angle,
        fill: MITOCHONDRION.gamma_multiply(alpha),
        stroke: Stroke::new((r * 0.02).max(0.9), MITO_EDGE.gamma_multiply(alpha)),
    });
    painter.add(EllipseShape {
        center: c,
        radius: egui::vec2(len * 0.78, w * 0.7),
        angle,
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(0.7, Color32::from_rgb(255, 210, 160).gamma_multiply(alpha)),
    });
    // cristae: folds growing in from the inner membrane, alternately from each side
    for (j, x) in [-0.45, -0.15, 0.15, 0.45].into_iter().enumerate() {
        let side = if j % 2 == 0 { -1.0 } else { 1.0 };
        let from = rot(Pt::new(len * x, side * w * 0.68), angle);
        let to = rot(Pt::new(len * x, side * w * 0.1), angle);
        painter.line_segment(
            [c + egui::vec2(from.x, from.y), c + egui::vec2(to.x, to.y)],
            Stroke::new(0.8, MITO_EDGE.gamma_multiply(0.8 * alpha)),
        );
    }
}

/// The large close-up for the inspector, plus where to put each label.
pub(super) struct CloseUp {
    pub membrane: Pos2,
    pub cytoplasm: Pos2,
    pub nucleus: Option<Pos2>,
    pub mitochondrion: Option<Pos2>,
    pub strands: Option<Pos2>,
}

/// Draws `cell` big, centred at `centre` with the given radius in pixels.
pub(super) fn draw_close_up(
    painter: &egui::Painter,
    centre: Pos2,
    radius: f32,
    cell: &Cell,
    time: f32,
) -> CloseUp {
    let look = Look {
        centre: Pt::new(0.0, 0.0),
        nucleus_alpha: 1.0,
        ..Look::default()
    };
    draw_body(
        painter,
        centre,
        cell,
        radius,
        &look,
        time,
        &Counter::new(usize::MAX),
    );
    let spec = viz::bilayer_spec(radius);
    let at_polar = |a: f32, d: f32| centre + egui::vec2(a.cos(), a.sin()) * d;
    let mito = (cell.mitochondria > 0).then(|| {
        let (slot, _) = viz::mito_slot(0, radius);
        centre + egui::vec2(slot.x, slot.y)
    });
    CloseUp {
        membrane: at_polar(-0.8, spec.mid),
        cytoplasm: at_polar(2.5, (spec.inner_edge + viz::nucleus_radius(radius)) * 0.5),
        nucleus: cell
            .nucleus
            .then(|| at_polar(-2.0, viz::nucleus_radius(radius) * 0.6)),
        mitochondrion: mito,
        strands: (!cell.strands.is_empty()).then(|| {
            let area = if cell.nucleus {
                viz::nucleus_radius(radius) * 0.55
            } else {
                spec.inner_edge * 0.6
            };
            at_polar(0.0, area * 0.3)
        }),
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
    let mut s = format!(
        "{kind} #{}\nGeneration: {}\nFitness: {:.2}\n",
        c.id,
        c.generation,
        c.fitness()
    );
    s += &format!(
        "Strands: {}   Lipids: {}   Nucleotides: {}\n",
        c.strands.len(),
        c.lipids,
        c.nucleotides
    );
    if c.nucleus {
        s += "Nucleus: yes (strands last longer and copy better)\n";
    }
    if c.mitochondria > 0 {
        s += &format!(
            "Mitochondria: {} (energy x{:.2})\n",
            c.mitochondria,
            c.energy()
        );
    }
    if let Some(colony) = c.colony {
        s += &format!("Colony {colony}, role: {}\n", c.role.label());
    }
    if let Some(parent) = c.parent {
        s += &format!("Divided from cell {parent}\n");
    }
    s += "\nClick to inspect";
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tooltip_tells_the_cells_story() {
        let mut c = Cell::new(7, 0.0, 0.0, 0);
        c.lipids = 5;
        let plain = cell_tooltip(&c);
        assert!(
            plain.starts_with("Cell #7") && !plain.contains("Nucleus") && !plain.contains("Colony")
        );
        c.nucleus = true;
        c.mitochondria = 2;
        c.colony = Some(3);
        c.role = Role::Soma;
        c.parent = Some(2);
        let full = cell_tooltip(&c);
        for needle in [
            "Eukaryote #7",
            "Nucleus: yes",
            "Mitochondria: 2",
            "Colony 3, role: soma",
            "Divided from cell 2",
            "Click to inspect",
        ] {
            assert!(full.contains(needle), "missing {needle} in {full}");
        }
    }

    #[test]
    fn roles_get_colour_hints_and_generalists_none() {
        assert!(role_tint(Role::Generalist).is_none());
        let tints: Vec<_> = [Role::Germ, Role::Soma, Role::Feeder]
            .into_iter()
            .map(|r| role_tint(r).unwrap())
            .collect();
        assert!(tints[0] != tints[1] && tints[1] != tints[2] && tints[0] != tints[2]);
        assert_eq!(mix(Color32::BLACK, Color32::WHITE, 0.0), Color32::BLACK);
    }

    #[test]
    fn rotating_a_point_keeps_its_length() {
        let p = rot(Pt::new(3.0, 4.0), 1.1);
        assert!((p.len() - 5.0).abs() < 1e-4);
    }
}
