//! The "Genes and Proteins" screen: draws the pure timeline in `crate::formation`.
//! Nothing here decides what happens, it only turns a `Snapshot` into shapes.

use super::molecule_draw::base_color;
use super::{MENU_BG, Screen, back_button};
use crate::chain_view::residue_hue;
use crate::formation::{Formation, MAX_SPEED, MIN_SPEED, Phase, Snapshot, amino_name, random_gene};
use crate::model::Codon;
use crate::soup::gene_boost;
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, vec2};
use std::time::{Duration, Instant};

const BOND: Color32 = Color32::from_rgb(190, 200, 215);
const HYDROGEN: Color32 = Color32::from_rgb(255, 255, 255);
const RIBOSOME: Color32 = Color32::from_rgb(120, 110, 200);
const TEXT: Color32 = Color32::from_rgb(230, 235, 240);
/// Gap between the two strands when they are paired.
const GAP: f32 = 64.0;
/// How far each strand moves away when unzipped.
const PULL: f32 = 52.0;
const AMINO_RADIUS: f32 = 13.0;
const AMINO_SPACING: f32 = 30.0;

pub struct FormationView {
    scene: Formation,
    last_tick: Instant,
}

impl Default for FormationView {
    fn default() -> Self {
        Self {
            scene: Formation::default(),
            last_tick: Instant::now(),
        }
    }
}

impl FormationView {
    pub(super) fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let now = Instant::now();
        self.scene
            .advance(now.duration_since(self.last_tick).as_secs_f32().min(0.25));
        self.last_tick = now;

        let next = self.controls_ui(ui);
        egui::Panel::bottom("facts").show(ui, |ui| self.facts_ui(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(MENU_BG))
            .show(ui, |ui| self.canvas_ui(ui));

        if self.scene.paused {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        } else {
            ui.ctx().request_repaint();
        }
        next
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let mut next = None;
        egui::Panel::top("formation_controls").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if back_button(ui) {
                    next = Some(Screen::Menu);
                }
                ui.separator();
                if ui
                    .button(if self.scene.paused { "Play" } else { "Pause" })
                    .clicked()
                {
                    self.scene.paused = !self.scene.paused;
                }
                if ui.button("Restart").clicked() {
                    self.scene.restart();
                }
                let mut speed = self.scene.speed();
                if ui
                    .add(
                        egui::Slider::new(&mut speed, MIN_SPEED..=MAX_SPEED)
                            .logarithmic(true)
                            .suffix("x")
                            .text("Speed"),
                    )
                    .changed()
                {
                    self.scene.set_speed(speed);
                }
                ui.separator();
                ui.selectable_value(&mut self.scene.dna, false, "RNA (U)");
                ui.selectable_value(&mut self.scene.dna, true, "DNA (T)");
                ui.separator();
                if ui.button("New gene").clicked() {
                    let gene = random_gene(&mut rand::rng());
                    self.scene.set_gene(&gene);
                }
            });
            ui.horizontal_wrapped(|ui| {
                let current = self.scene.locate().0;
                for phase in Phase::ALL.into_iter().filter(|p| *p != Phase::Rest) {
                    if ui
                        .selectable_label(phase == current, phase.title())
                        .clicked()
                    {
                        self.scene.seek(phase);
                    }
                }
            });
        });
        next
    }

    /// The caption, the gene with its reading frame, the protein and its fitness.
    fn facts_ui(&self, ui: &mut egui::Ui) {
        let snap = self.scene.snapshot();
        ui.strong(snap.phase.title());
        ui.label(snap.phase.caption());
        if self.scene.dna && snap.phase == Phase::Translate {
            ui.weak("The DNA gene was first copied into messenger RNA, so T reads as U here.");
        }
        let frame = self.scene.frame();
        ui.horizontal_wrapped(|ui| {
            ui.monospace("Gene: ");
            for (k, c) in self.scene.gene().chars().enumerate() {
                let reading = k >= frame.start && k < frame.start + frame.codons.len() * 3;
                let letter = self.scene.shown(Phase::Assemble, c);
                let text = egui::RichText::new(letter.to_string())
                    .monospace()
                    .color(base_color(c));
                ui.label(if reading {
                    text.underline().strong()
                } else {
                    text.weak()
                });
            }
        });
        let protein = self.scene.protein();
        let fitness = self.scene.fitness();
        ui.label(format!(
            "Protein: {protein}   fitness {fitness:.2}   a strand with this gene copies itself {:.2}x as fast",
            gene_boost(self.scene.gene())
        ));
        ui.add(
            egui::ProgressBar::new(fitness as f32)
                .desired_width(220.0)
                .text("protein fitness"),
        );
    }

    fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let rect = response.rect;
        let snap = self.scene.snapshot();
        let layout = Layout::new(rect, self.scene.gene().len());
        match snap.phase {
            Phase::Assemble => self.draw_assemble(&painter, &layout, &snap),
            Phase::Pair => self.draw_pair(&painter, &layout, &snap),
            Phase::Twist | Phase::Unzip => self.draw_helix(&painter, &layout, &snap),
            Phase::Translate | Phase::Fold | Phase::Rest => {
                self.draw_translate(&painter, &layout, &snap)
            }
        }
    }

    fn bead(
        &self,
        painter: &egui::Painter,
        layout: &Layout,
        phase: Phase,
        pos: Pos2,
        base: char,
        look: Look,
    ) {
        let Look { alpha, scale } = look;
        let r = layout.radius * scale;
        let color = base_color(base);
        painter.circle_filled(pos, r * 1.25, color.gamma_multiply(0.18 * alpha));
        painter.circle_filled(pos, r, color.gamma_multiply(alpha));
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            self.scene.shown(phase, base),
            FontId::monospace(r * 1.15),
            Color32::from_rgb(20, 24, 30).gamma_multiply(alpha),
        );
    }

    /// Row of the gene's bases at height `y`, all in place.
    fn backbone(&self, painter: &egui::Painter, layout: &Layout, y: f32, count: usize, alpha: f32) {
        for k in 1..count {
            painter.line_segment(
                [
                    Pos2::new(layout.x(k as f32 - 1.0), y),
                    Pos2::new(layout.x(k as f32), y),
                ],
                Stroke::new(3.0, BOND.gamma_multiply(alpha)),
            );
        }
    }

    fn draw_assemble(&self, painter: &egui::Painter, layout: &Layout, snap: &Snapshot) {
        let y = layout.rect.top() + layout.rect.height() * 0.34;
        let time = self.scene.time();
        let gene: Vec<char> = self.scene.gene().chars().collect();
        let in_place = (snap.joined.floor() as usize).min(gene.len());
        self.backbone(painter, layout, y, in_place, 1.0);
        for (k, &base) in gene.iter().enumerate() {
            let arrived = (snap.joined - k as f32).clamp(0.0, 1.0);
            let slot = Pos2::new(layout.x(k as f32), y);
            let (sx, sy) = scatter(k, layout.rect);
            let wobble = vec2(
                (time * 1.3 + k as f32).sin() * 7.0,
                (time * 1.7 + k as f32 * 2.0).cos() * 7.0,
            ) * (1.0 - arrived);
            let free = Pos2::new(sx + wobble.x, sy + wobble.y);
            let pos = free.lerp(slot, ease(arrived));
            if k == in_place && arrived > 0.0 && k > 0 {
                // the bond forming
                painter.line_segment(
                    [Pos2::new(layout.x(k as f32 - 1.0), y), pos],
                    Stroke::new(2.0, BOND.gamma_multiply(arrived)),
                );
            }
            self.bead(painter, layout, snap.phase, pos, base, Look::PLAIN);
        }
        label(
            painter,
            Pos2::new(layout.rect.center().x, layout.rect.top() + 24.0),
            "free nucleotides bond into a strand",
        );
    }

    fn draw_pair(&self, painter: &egui::Painter, layout: &Layout, snap: &Snapshot) {
        let y = layout.rect.top() + layout.rect.height() * 0.34;
        let gene: Vec<char> = self.scene.gene().chars().collect();
        self.backbone(painter, layout, y, gene.len(), 1.0);
        let docked_whole = (snap.docked.floor() as usize).min(gene.len());
        self.backbone(painter, layout, y + GAP, docked_whole, 1.0);
        for (k, &base) in gene.iter().enumerate() {
            self.bead(
                painter,
                layout,
                snap.phase,
                Pos2::new(layout.x(k as f32), y),
                base,
                Look::PLAIN,
            );
            let d = (snap.docked - k as f32).clamp(0.0, 1.0);
            if d <= 0.0 {
                continue;
            }
            let partner = crate::model::complement(base);
            let from_below = Pos2::new(
                layout.x(k as f32) + 18.0 * (1.0 - d),
                y + GAP + 70.0 * (1.0 - ease(d)),
            );
            let pos = if d >= 1.0 {
                Pos2::new(layout.x(k as f32), y + GAP)
            } else {
                from_below
            };
            self.bead(
                painter,
                layout,
                snap.phase,
                pos,
                partner,
                Look {
                    alpha: d.min(1.0),
                    scale: 1.0,
                },
            );
            if d >= 0.85 {
                Self::bonds(
                    painter,
                    layout,
                    layout.x(k as f32),
                    (y, y + GAP),
                    Formation::bonds_between(base),
                    (d - 0.85) / 0.15,
                );
            }
        }
        label(
            painter,
            Pos2::new(layout.rect.center().x, layout.rect.top() + 24.0),
            "A pairs with U (T in DNA), C pairs with G",
        );
    }

    /// Hydrogen-bond dashes between two beads at the same x.
    fn bonds(
        painter: &egui::Painter,
        layout: &Layout,
        x: f32,
        (y1, y2): (f32, f32),
        count: usize,
        alpha: f32,
    ) {
        let (top, bottom) = (y1 + layout.radius + 2.0, y2 - layout.radius - 2.0);
        if bottom <= top {
            return;
        }
        let gap = (layout.spacing * 0.16).max(2.0);
        for i in 0..count {
            let dx = (i as f32 - (count as f32 - 1.0) / 2.0) * gap;
            let mut y = top;
            while y < bottom {
                painter.line_segment(
                    [
                        Pos2::new(x + dx, y),
                        Pos2::new(x + dx, (y + 3.0).min(bottom)),
                    ],
                    Stroke::new(1.3, HYDROGEN.gamma_multiply(alpha.clamp(0.0, 1.0))),
                );
                y += 6.0;
            }
        }
    }

    fn draw_helix(&self, painter: &egui::Painter, layout: &Layout, snap: &Snapshot) {
        let n = self.scene.gene().len();
        let gene: Vec<char> = self.scene.gene().chars().collect();
        let y0 = layout.rect.top() + layout.rect.height() * 0.34;
        let centre = y0 + GAP / 2.0;
        let amp = GAP / 2.0;
        let mut items: Vec<(f32, Pos2, char, f32, usize, bool)> = Vec::with_capacity(n * 2);
        for (k, base) in gene.iter().enumerate() {
            let theta = k as f32 * 0.55 * snap.twist;
            let off = ease((snap.unzipped - k as f32).clamp(0.0, 1.0));
            let wave = theta.cos() * amp;
            let depth = theta.sin();
            let scale = 1.0 + 0.22 * depth * snap.twist.min(1.0);
            let top = Pos2::new(layout.x(k as f32), centre + wave - off * PULL);
            let bottom = Pos2::new(layout.x(k as f32), centre - wave + off * PULL);
            items.push((depth, top, *base, scale, k, true));
            items.push((
                -depth,
                bottom,
                crate::model::complement(*base),
                1.0 - 0.22 * depth * snap.twist.min(1.0),
                k,
                false,
            ));
            // rungs fade as the strands part
            let rung = (1.0 - off) * (0.35 + 0.65 * theta.cos().abs());
            if rung > 0.05 {
                Self::bonds(
                    painter,
                    layout,
                    layout.x(k as f32),
                    (
                        top.y.min(bottom.y) - layout.radius,
                        top.y.max(bottom.y) + layout.radius,
                    ),
                    Formation::bonds_between(*base),
                    rung,
                );
            }
        }
        // backbones follow their beads
        for k in 1..n {
            for upper in [true, false] {
                let pick = |kk: usize| {
                    items
                        .iter()
                        .find(|i| i.4 == kk && i.5 == upper)
                        .map(|i| i.1)
                        .unwrap_or_default()
                };
                painter.line_segment([pick(k - 1), pick(k)], Stroke::new(3.0, BOND));
            }
        }
        items.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, pos, base, scale, _, _) in items {
            self.bead(
                painter,
                layout,
                snap.phase,
                pos,
                base,
                Look { alpha: 1.0, scale },
            );
        }
        let text = if snap.phase == Phase::Twist {
            "the paired strands twist into a double helix"
        } else {
            "the helix unzips into two strands"
        };
        label(
            painter,
            Pos2::new(layout.rect.center().x, layout.rect.top() + 24.0),
            text,
        );
    }

    fn draw_translate(&self, painter: &egui::Painter, layout: &Layout, snap: &Snapshot) {
        let rect = layout.rect;
        let y = rect.top() + rect.height() * 0.66;
        let gene: Vec<char> = self.scene.gene().chars().collect();
        let frame = self.scene.frame();
        self.backbone(painter, layout, y, gene.len(), 1.0);
        for (k, &base) in gene.iter().enumerate() {
            self.bead(
                painter,
                layout,
                snap.phase,
                Pos2::new(layout.x(k as f32), y),
                base,
                Look::PLAIN,
            );
        }
        // the start codon and the stop codon get a marker
        let mark = |k: usize, text: &str, color: Color32| {
            painter.text(
                Pos2::new(layout.x(k as f32 + 1.0), y + layout.radius + 16.0),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(12.0),
                color,
            );
        };
        mark(frame.start, "start", Color32::from_rgb(120, 230, 150));
        if frame.is_closed() {
            mark(
                frame.start + (frame.codons.len() - 1) * 3,
                "stop",
                Color32::from_rgb(255, 120, 120),
            );
        }

        if snap.phase == Phase::Translate && !snap.released {
            self.draw_ribosome(painter, layout, snap, y);
        }

        // the growing chain
        let protein: Vec<char> = self.scene.protein().chars().collect();
        let chain_y = rect.top() + rect.height() * 0.28;
        let growing = snap.phase == Phase::Translate && !snap.released;
        let count = if growing {
            // a bead per whole amino acid, plus the one being made, growing in
            let partial = match &snap.codon {
                Some((_, Codon::Amino(_))) => snap.codons_read.fract(),
                _ => 0.0,
            };
            (snap.amino_acids as f32 + partial).min(protein.len() as f32)
        } else {
            protein.len() as f32
        };
        let line_x = |i: usize| {
            rect.center().x + (i as f32 - (protein.len() as f32 - 1.0) / 2.0) * AMINO_SPACING
        };
        let coil = |i: usize| {
            let m = protein.len().max(1) as f32;
            let radius = (m * 31.0 / std::f32::consts::TAU).max(26.0);
            let angle = i as f32 / m * std::f32::consts::TAU - 1.2;
            let wobble = if i.is_multiple_of(2) { 5.0 } else { -5.0 };
            Pos2::new(
                rect.center().x + (radius + wobble) * angle.cos(),
                chain_y + 6.0 + (radius * 0.9 + wobble) * angle.sin(),
            )
        };
        let fold = ease(snap.fold);
        let position = |i: usize| Pos2::new(line_x(i), chain_y).lerp(coil(i), fold);
        let shown = count.ceil() as usize;
        for i in 1..shown.min(protein.len()) {
            painter.line_segment([position(i - 1), position(i)], Stroke::new(3.5, BOND));
        }
        if growing && shown > 0 && snap.phase == Phase::Translate {
            let exit = Pos2::new(layout.x(snap.ribosome_at + 1.0), y - layout.radius - 70.0);
            painter.line_segment(
                [exit, position(shown - 1)],
                Stroke::new(1.5, BOND.gamma_multiply(0.5)),
            );
        }
        for (i, &letter) in protein.iter().enumerate().take(shown) {
            let scale = (count - i as f32).clamp(0.0, 1.0);
            let color: Color32 =
                egui::ecolor::Hsva::new(residue_hue(letter), 0.55, 0.95, 1.0).into();
            let pos = position(i);
            painter.circle_filled(pos, AMINO_RADIUS * 1.25 * scale, color.gamma_multiply(0.2));
            painter.circle_filled(pos, AMINO_RADIUS * scale, color);
            painter.text(
                pos,
                Align2::CENTER_CENTER,
                letter,
                FontId::monospace(15.0 * scale),
                Color32::from_rgb(20, 24, 30),
            );
        }
        if snap.phase == Phase::Fold || snap.phase == Phase::Rest {
            let text = format!(
                "{}   fitness {:.2}",
                self.scene.protein(),
                self.scene.fitness()
            );
            label(painter, Pos2::new(rect.center().x, chain_y - 70.0), &text);
            if snap.phase == Phase::Fold {
                label(
                    painter,
                    Pos2::new(rect.center().x, chain_y - 48.0),
                    if snap.fold < 0.99 {
                        "released at the stop codon, folding"
                    } else {
                        "folded"
                    },
                );
            }
        } else {
            label(
                painter,
                Pos2::new(rect.center().x, rect.top() + 24.0),
                "the ribosome reads the strand and builds a protein",
            );
        }
    }

    fn draw_ribosome(&self, painter: &egui::Painter, layout: &Layout, snap: &Snapshot, y: f32) {
        let centre_x = layout.x(snap.ribosome_at + 1.0);
        let width = layout.spacing * 3.2;
        let big = Rect::from_center_size(
            Pos2::new(centre_x, y - layout.radius - 22.0),
            vec2(width, 38.0),
        );
        let small = Rect::from_center_size(
            Pos2::new(centre_x, y + layout.radius + 7.0),
            vec2(width * 0.9, 18.0),
        );
        painter.rect_filled(big, 16.0, RIBOSOME.gamma_multiply(0.75));
        painter.rect_stroke(
            big,
            16.0,
            Stroke::new(1.5, RIBOSOME),
            egui::StrokeKind::Outside,
        );
        painter.rect_filled(small, 9.0, RIBOSOME.gamma_multiply(0.5));
        match &snap.codon {
            Some((codon, meaning)) => {
                let (what, color) = match meaning {
                    Codon::Amino(a) => (format!("{a}  {}", amino_name(*a)), TEXT),
                    Codon::Stop => ("STOP".to_string(), Color32::from_rgb(255, 120, 120)),
                };
                painter.text(
                    big.center(),
                    Align2::CENTER_CENTER,
                    codon,
                    FontId::monospace(16.0),
                    Color32::WHITE,
                );
                painter.text(
                    Pos2::new(centre_x, big.top() - 14.0),
                    Align2::CENTER_CENTER,
                    what,
                    FontId::proportional(14.0),
                    color,
                );
            }
            None => {
                painter.text(
                    big.center(),
                    Align2::CENTER_CENTER,
                    "scanning",
                    FontId::proportional(12.0),
                    TEXT,
                );
            }
        }
    }
}

/// How a bead is drawn: how opaque, and how big compared to normal.
#[derive(Clone, Copy)]
struct Look {
    alpha: f32,
    scale: f32,
}

impl Look {
    const PLAIN: Look = Look {
        alpha: 1.0,
        scale: 1.0,
    };
}

/// Where the strand's beads sit on screen.
struct Layout {
    rect: Rect,
    spacing: f32,
    radius: f32,
    first: f32,
}

impl Layout {
    fn new(rect: Rect, bases: usize) -> Self {
        let spacing = ((rect.width() - 80.0) / bases.max(1) as f32).clamp(14.0, 34.0);
        let first = rect.center().x - (bases.saturating_sub(1)) as f32 * spacing / 2.0;
        Self {
            rect,
            spacing,
            radius: (spacing * 0.42).min(12.0),
            first,
        }
    }

    /// The x of base `index` (fractional indices fall between beads).
    fn x(&self, index: f32) -> f32 {
        self.first + index * self.spacing
    }
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A fixed, scattered starting place for bead `k`, in the lower part of the canvas.
fn scatter(k: usize, rect: Rect) -> (f32, f32) {
    let h = |seed: f32| ((seed.sin() * 43_758.547).fract()).abs();
    let (a, b) = (h(k as f32 * 12.989 + 1.0), h(k as f32 * 78.233 + 2.0));
    (
        rect.left() + 40.0 + a * (rect.width() - 80.0),
        rect.top() + rect.height() * (0.55 + 0.35 * b),
    )
}

fn label(painter: &egui::Painter, pos: Pos2, text: &str) {
    painter.text(
        pos,
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(15.0),
        TEXT.gamma_multiply(0.8),
    );
}
