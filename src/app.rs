//! The GUI: a launcher menu, the molecule simulation and the chain generator.

use crate::model::{Molecule, random_chains};
use crate::soup::{PrimordialSoup, TARGET_PATTERN};
use eframe::egui::{self, Color32, Pos2, RichText, Sense, Stroke, Vec2};
use std::collections::HashSet;
use std::time::{Duration, Instant};

const STEPS_PER_SECOND: f64 = 60.0;
const BASE_MOLECULE_SIZE: f32 = 6.0;
const CHAIN_COUNT: usize = 20;
const MENU_BG: Color32 = Color32::from_rgb(0x18, 0x1c, 0x1f);
const SOUP_BG: Color32 = Color32::from_rgb(10, 10, 20);
const CHAIN_TEXT: Color32 = Color32::from_rgb(0xb3, 0xe5, 0xfc);

#[derive(Default)]
pub struct App {
    screen: Screen,
}

/// Which screen the app opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Start {
    #[default]
    Menu,
    Molecules,
    Chains,
}

impl App {
    pub fn new(start: Start) -> Self {
        let screen = match start {
            Start::Menu => Screen::Menu,
            Start::Molecules => Screen::Soup(Box::default()),
            Start::Chains => Screen::Chains(ChainsView::default()),
        };
        Self { screen }
    }
}

#[derive(Default)]
enum Screen {
    #[default]
    Menu,
    Soup(Box<SoupView>),
    Chains(ChainsView),
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let next = match &mut self.screen {
            Screen::Menu => menu(ui),
            Screen::Soup(view) => view.ui(ui),
            Screen::Chains(view) => view.ui(ui),
        };
        if let Some(next) = next {
            self.screen = next;
        }
    }
}

fn back_button(ui: &mut egui::Ui) -> bool {
    ui.button("Back to menu").clicked()
}

fn menu(ui: &mut egui::Ui) -> Option<Screen> {
    let mut next = None;
    egui::CentralPanel::default().frame(egui::Frame::new().fill(MENU_BG)).show(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space((ui.available_height() / 2.0 - 90.0).max(20.0));
            ui.heading(RichText::new("Abiogenesis Simulation").size(26.0).color(Color32::WHITE));
            ui.add_space(24.0);
            let size = Vec2::new(280.0, 40.0);
            if ui.add_sized(size, egui::Button::new("Molecule Simulation")).clicked() {
                next = Some(Screen::Soup(Box::default()));
            }
            ui.add_space(12.0);
            if ui.add_sized(size, egui::Button::new("Amino Acid Chain Simulation")).clicked() {
                next = Some(Screen::Chains(ChainsView::default()));
            }
        });
    });
    next
}

pub struct ChainsView {
    chains: Vec<String>,
}

impl Default for ChainsView {
    fn default() -> Self {
        Self { chains: random_chains(&mut rand::rng(), CHAIN_COUNT) }
    }
}

impl ChainsView {
    fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let mut next = None;
        egui::CentralPanel::default().frame(egui::Frame::new().fill(MENU_BG).inner_margin(24.0)).show(ui, |ui| {
            ui.horizontal(|ui| {
                if back_button(ui) {
                    next = Some(Screen::Menu);
                }
                ui.label(RichText::new("Amino Acid Chain Simulation").size(20.0).color(Color32::WHITE));
            });
            ui.add_space(10.0);
            if ui.button("Regenerate Chains").clicked() {
                *self = Self::default();
            }
            ui.add_space(10.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                for chain in &self.chains {
                    ui.label(RichText::new(chain).monospace().size(16.0).color(CHAIN_TEXT));
                }
            });
        });
        next
    }
}

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

    fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        self.advance();
        self.fps.frame();
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
                    let (w, h) = (self.soup.width(), self.soup.height());
                    *self = Self::default();
                    self.soup.resize(w, h);
                }
            });
        });

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

    fn stats_ui(&self, ui: &mut egui::Ui) {
        ui.heading("Statistics");
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
        ui.add_space(8.0);
        ui.strong("Molecule counts");
        egui::ScrollArea::vertical().id_salt("counts").max_height(220.0).show(ui, |ui| {
            for (name, count) in self.soup.counts() {
                ui.label(format!("{name}: {count}"));
            }
        });
        ui.add_space(8.0);
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
        let (w, h) = (rect.width() as f64, rect.height() as f64);
        if (w - self.soup.width()).abs() > 0.5 || (h - self.soup.height()).abs() > 0.5 {
            self.soup.resize(w, h);
        }

        let to_screen = |m: &Molecule| Pos2::new(rect.left() + m.position.x as f32, rect.top() + m.position.y as f32);
        for m in &self.soup.molecules {
            let center = to_screen(m);
            let size = molecule_size(m);
            let color = molecule_color(m);
            painter.circle_filled(center, size * 0.7, color.gamma_multiply(0.3));
            painter.circle_filled(center, size * 0.5, color);
        }

        let hovered = response
            .hover_pos()
            .and_then(|p| self.soup.molecules.iter().find(|m| to_screen(m).distance(p) < molecule_size(m)));
        if let Some(m) = hovered {
            painter.circle_stroke(to_screen(m), molecule_size(m) * 0.7, Stroke::new(1.0, Color32::WHITE));
            let text = tooltip_text(m);
            response.on_hover_ui_at_pointer(|ui| {
                ui.monospace(text);
            });
        }
    }
}

fn molecule_size(m: &Molecule) -> f32 {
    BASE_MOLECULE_SIZE + m.energy as f32 * 2.0
}

fn molecule_color(m: &Molecule) -> Color32 {
    match (m.sequence(), m.name().as_str()) {
        // Chains are coloured by length.
        (Some(seq), _) => egui::ecolor::Hsva::new(((seq.len() * 20) % 360) as f32 / 360.0, 0.8, 1.0, 1.0).into(),
        (None, "H2O") => Color32::from_rgb(0, 255, 255),
        (None, "CH4") => Color32::from_rgb(144, 238, 144),
        (None, "CH3OH") => Color32::from_rgb(128, 0, 128),
        _ => Color32::WHITE,
    }
}

fn tooltip_text(m: &Molecule) -> String {
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

/// Frames per second, averaged over one-second windows.
struct Fps {
    window_start: Instant,
    frames: u32,
    value: f64,
}

impl Default for Fps {
    fn default() -> Self {
        Self { window_start: Instant::now(), frames: 0, value: 0.0 }
    }
}

impl Fps {
    fn frame(&mut self) {
        self.frames += 1;
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed >= 1.0 {
            self.value = self.frames as f64 / elapsed;
            self.frames = 0;
            self.window_start = Instant::now();
        }
    }
}
