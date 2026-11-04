//! The molecule simulation screen: a live soup, its statistics and its canvas.

use super::fps::Fps;
use super::molecule_draw::{CATALYST_COLOR, LIPID_COLOR, base_color, molecule_color, molecule_size, tooltip_text};
use super::{Screen, back_button};
use crate::environment::{Environment, Preset};
use crate::model::Molecule;
use crate::soup::{PrimordialSoup, TARGET_PATTERN};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const STEPS_PER_SECOND: f64 = 60.0;
const SOUP_BG: Color32 = Color32::from_rgb(10, 10, 20);
const WATER: Color32 = Color32::from_rgba_premultiplied(8, 22, 34, 50);
const SURFACE: Color32 = Color32::from_rgb(70, 150, 200);
const DROPLET: Color32 = Color32::from_rgb(230, 90, 190);

pub struct SoupView {
    soup: PrimordialSoup,
    running: bool,
    last_tick: Instant,
    pending: f64,
    steps: u64,
    fps: Fps,
    seen_patterns: HashSet<String>,
    log: Vec<String>,
    preset: Preset,
    base_temperature: f64,
    base_ph: f64,
}

impl Default for SoupView {
    fn default() -> Self {
        Self {
            soup: PrimordialSoup::standard(800.0, 600.0),
            running: true,
            last_tick: Instant::now(),
            pending: 0.0,
            steps: 0,
            fps: Fps::default(),
            seen_patterns: HashSet::new(),
            log: Vec::new(),
            preset: Preset::default(),
            base_temperature: 300.0,
            base_ph: 7.0,
        }
    }
}

impl SoupView {
    fn step(&mut self) {
        self.steps += 1;
        for seq in self.soup.simulate_step() {
            if self.seen_patterns.insert(seq.clone()) {
                let line = format!("step {}: '{TARGET_PATTERN}' found in {seq}", self.steps);
                println!("{line}");
                self.log.push(line);
            }
        }
    }

    /// Runs the simulation at a fixed 60 steps per second, whatever the frame rate.
    fn advance(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f64().min(0.25);
        self.last_tick = now;
        if !self.running {
            return;
        }
        self.pending += elapsed * STEPS_PER_SECOND;
        while self.pending >= 1.0 {
            self.step();
            self.pending -= 1.0;
        }
    }

    /// Hands the chosen preset and base conditions to the soup.
    fn apply_environment(&mut self) {
        self.soup.set_environment(Environment::preset(self.preset, self.base_temperature, self.base_ph));
    }

    /// Starts a fresh soup, keeping the current canvas size and environment choice.
    fn reset(&mut self) {
        let (w, h) = (self.soup.width(), self.soup.height());
        let (preset, temperature, ph) = (self.preset, self.base_temperature, self.base_ph);
        *self = Self { preset, base_temperature: temperature, base_ph: ph, ..Self::default() };
        self.soup.resize(w, h);
        self.apply_environment();
    }

    pub(super) fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        self.advance();
        self.fps.frame();
        let next = self.controls_ui(ui);

        egui::Panel::right("stats").resizable(false).exact_size(220.0).show(ui, |ui| {
            self.stats_ui(ui);
        });

        egui::CentralPanel::default().frame(egui::Frame::new().fill(SOUP_BG)).show(ui, |ui| {
            self.canvas_ui(ui);
        });

        if self.running {
            ui.ctx().request_repaint();
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
        next
    }

    /// The top bar: back, pause/resume, step and reset.
    fn controls_ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let mut next = None;
        egui::Panel::top("controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                if back_button(ui) {
                    next = Some(Screen::Menu);
                }
                ui.separator();
                if ui.button(if self.running { "Pause" } else { "Resume" }).clicked() {
                    self.running = !self.running;
                }
                if ui.add_enabled(!self.running, egui::Button::new("Step")).clicked() {
                    self.step();
                }
                if ui.button("Reset").clicked() {
                    self.reset();
                }
            });
            self.environment_ui(ui);
        });
        next
    }

    /// The environment row: a preset, base temperature and pH, and what the soup feels right now.
    fn environment_ui(&mut self, ui: &mut egui::Ui) {
        let mut changed = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("Environment");
            egui::ComboBox::from_id_salt("preset").selected_text(self.preset.label()).show_ui(ui, |ui| {
                for p in Preset::ALL {
                    changed |= ui.selectable_value(&mut self.preset, p, p.label()).changed();
                }
            });
            changed |= ui.add(egui::Slider::new(&mut self.base_temperature, 250.0..=420.0).suffix(" K").text("Temperature")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.base_ph, 1.0..=13.0).text("pH")).changed();
        });
        if changed {
            self.apply_environment();
        }
        ui.horizontal(|ui| {
            let s = &self.soup;
            ui.weak(format!(
                "Now: {:.0} K, pH {:.1}, pool {:.0}% full{}",
                s.temperature,
                s.ph,
                s.wetness * 100.0,
                if s.protocells.is_empty() { String::new() } else { format!(", {} protocell(s)", s.protocells.len()) },
            ));
        });
    }

    fn stats_ui(&self, ui: &mut egui::Ui) {
        ui.heading("Statistics");
        self.stats_grid(ui);
        ui.add_space(8.0);
        ui.strong("Molecule counts");
        egui::ScrollArea::vertical().id_salt("counts").max_height(220.0).show(ui, |ui| {
            for (name, count) in self.soup.counts() {
                ui.label(format!("{name}: {count}"));
            }
        });
        ui.add_space(8.0);
        self.sightings_ui(ui);
    }

    fn stats_grid(&self, ui: &mut egui::Ui) {
        egui::Grid::new("stats_grid").num_columns(2).striped(true).show(ui, |ui| {
            let mut row = |k: &str, v: String| {
                ui.label(k);
                ui.label(v);
                ui.end_row();
            };
            row("FPS", format!("{:.1}", self.fps.value));
            row("Step", self.steps.to_string());
            row("Molecules", self.soup.molecules.len().to_string());
            row("Chains", self.soup.amino_acid_count().to_string());
            row("Temperature", format!("{:.1} K", self.soup.temperature));
            row("pH", format!("{:.1}", self.soup.ph));
            row("Pool", format!("{:.0}% full", self.soup.wetness * 100.0));
            row("Protocells", self.soup.protocells.len().to_string());
            row("In droplets", self.soup.chains_in_protocells().to_string());
            row("Formed / burst", format!("{} / {}", self.soup.protocells_formed(), self.soup.protocells_burst()));
            row("Nucleotides", self.soup.nucleotide_count().to_string());
            row("RNA strands", format!("{} ({} catalysts)", self.soup.strand_count(), self.soup.catalyst_count()));
            row("Longest strand", self.soup.longest_strand().to_string());
            row("Strand fitness", format!("{:.2}", self.soup.mean_strand_fitness()));
            row("Copied / decayed", format!("{} / {}", self.soup.strand_copies(), self.soup.strands_decayed()));
            row("Lipids", self.soup.lipid_count().to_string());
            row("Reactions", self.soup.total_reactions().to_string());
            row("This step", self.soup.reactions_this_step().to_string());
        });
    }

    /// The log of steps where the target pattern first appeared in a sequence.
    fn sightings_ui(&self, ui: &mut egui::Ui) {
        ui.strong(format!("'{TARGET_PATTERN}' sightings"));
        if self.log.is_empty() {
            ui.weak("none yet");
        }
        egui::ScrollArea::vertical().id_salt("log").stick_to_bottom(true).show(ui, |ui| {
            for line in &self.log {
                ui.monospace(line);
            }
        });
    }

    fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let rect = response.rect;
        self.fit_to(rect);

        // the pool: everything below the water line
        let line = rect.top() + self.soup.water_line() as f32;
        painter.rect_filled(egui::Rect::from_min_max(Pos2::new(rect.left(), line), rect.right_bottom()), 0.0, WATER);
        if line > rect.top() {
            painter.line_segment([Pos2::new(rect.left(), line), Pos2::new(rect.right(), line)], Stroke::new(1.5, SURFACE));
        }
        for p in &self.soup.protocells {
            let center = Pos2::new(rect.left() + p.x as f32, rect.top() + p.y as f32);
            painter.circle_filled(center, p.radius as f32, DROPLET.gamma_multiply(0.12));
            painter.circle_stroke(center, p.radius as f32, Stroke::new(2.0, DROPLET));
        }

        for m in &self.soup.molecules {
            let center = screen_pos(rect, m);
            if let Some(seq) = m.strand_sequence() {
                draw_strand(&painter, center, seq, m.is_catalyst());
            } else if m.is_lipid() {
                draw_lipid(&painter, center);
            } else {
                let size = molecule_size(m);
                let color = molecule_color(m);
                painter.circle_filled(center, size * 0.7, color.gamma_multiply(0.3));
                painter.circle_filled(center, size * 0.5, color);
            }
        }

        let hovered = response
            .hover_pos()
            .and_then(|p| self.soup.molecules.iter().find(|m| screen_pos(rect, m).distance(p) < molecule_size(m)));
        if let Some(m) = hovered {
            painter.circle_stroke(screen_pos(rect, m), molecule_size(m) * 0.7, Stroke::new(1.0, Color32::WHITE));
            let text = tooltip_text(m);
            response.on_hover_ui_at_pointer(|ui| {
                ui.monospace(text);
            });
        }
    }

    /// Resizes the soup to the canvas when they differ by more than half a pixel.
    fn fit_to(&mut self, rect: egui::Rect) {
        let (w, h) = (rect.width() as f64, rect.height() as f64);
        if (w - self.soup.width()).abs() > 0.5 || (h - self.soup.height()).abs() > 0.5 {
            self.soup.resize(w, h);
        }
    }
}

/// A strand as a row of coloured beads, one per base (the first 12), with a gold
/// halo when it is a catalyst.
fn draw_strand(painter: &egui::Painter, center: Pos2, sequence: &str, catalyst: bool) {
    const SPACING: f32 = 4.5;
    let shown = sequence.chars().take(12).collect::<Vec<_>>();
    let left = center.x - (shown.len() as f32 - 1.0) * SPACING / 2.0;
    if catalyst {
        let half = shown.len() as f32 * SPACING / 2.0 + 3.0;
        painter.rect_filled(egui::Rect::from_center_size(center, egui::vec2(half * 2.0, 10.0)), 5.0, CATALYST_COLOR.gamma_multiply(0.25));
    }
    for (k, base) in shown.iter().enumerate() {
        painter.circle_filled(Pos2::new(left + k as f32 * SPACING, center.y), 2.4, base_color(*base));
    }
}

/// A lipid: a round head with two short tails.
fn draw_lipid(painter: &egui::Painter, center: Pos2) {
    for dx in [-1.5, 1.5] {
        painter.line_segment([center + egui::vec2(dx, 2.0), center + egui::vec2(dx * 1.6, 8.0)], Stroke::new(1.2, LIPID_COLOR.gamma_multiply(0.7)));
    }
    painter.circle_filled(center, 3.2, LIPID_COLOR);
}

fn screen_pos(rect: egui::Rect, m: &Molecule) -> Pos2 {
    Pos2::new(rect.left() + m.position.x as f32, rect.top() + m.position.y as f32)
}
