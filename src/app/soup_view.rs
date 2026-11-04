//! The molecule simulation screen: a live soup, its statistics and its canvas.

use super::fps::Fps;
use super::molecule_draw::{molecule_color, molecule_size, tooltip_text};
use super::{Screen, back_button};
use crate::model::Molecule;
use crate::soup::{PrimordialSoup, TARGET_PATTERN};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const STEPS_PER_SECOND: f64 = 60.0;
const SOUP_BG: Color32 = Color32::from_rgb(10, 10, 20);

pub struct SoupView {
    soup: PrimordialSoup,
    running: bool,
    last_tick: Instant,
    pending: f64,
    steps: u64,
    fps: Fps,
    seen_patterns: HashSet<String>,
    log: Vec<String>,
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

    /// Starts a fresh soup, keeping the current canvas size.
    fn reset(&mut self) {
        let (w, h) = (self.soup.width(), self.soup.height());
        *self = Self::default();
        self.soup.resize(w, h);
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
        });
        next
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

        for m in &self.soup.molecules {
            let center = screen_pos(rect, m);
            let size = molecule_size(m);
            let color = molecule_color(m);
            painter.circle_filled(center, size * 0.7, color.gamma_multiply(0.3));
            painter.circle_filled(center, size * 0.5, color);
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

fn screen_pos(rect: egui::Rect, m: &Molecule) -> Pos2 {
    Pos2::new(rect.left() + m.position.x as f32, rect.top() + m.position.y as f32)
}
