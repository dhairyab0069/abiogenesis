//! The chain build-up screen: a live soup (chains only) whose event log drives
//! a `ChainView`. This type only steps the soup and draws what the view-model says.

use super::tiles::{draw_row, legend_ui};
use super::{MENU_BG, Screen, back_button};
use crate::chain_view::ChainView;
use crate::events::ChainId;
use crate::soup::PrimordialSoup;
use eframe::egui;
use std::time::{Duration, Instant};

/// How fast the chains screen steps the soup. Slower than the molecule view's
/// 60: with the soup's own rates that is a few edits per second across the rows,
/// which is about what the eye can follow. A view-side setting; the soup's
/// generation, mutation and recombination rates are untouched.
const CHAIN_STEPS_PER_SECOND: f64 = 20.0;

pub struct ChainsView {
    soup: PrimordialSoup,
    view: ChainView,
    running: bool,
    last_tick: Instant,
    pending: f64,
}

impl Default for ChainsView {
    fn default() -> Self {
        let soup = PrimordialSoup::new(800.0, 600.0, 300.0, 7.0);
        let mut view = ChainView::new();
        view.sync(&soup.events, live_chains(&soup));
        Self {
            soup,
            view,
            running: true,
            last_tick: Instant::now(),
            pending: 0.0,
        }
    }
}

fn live_chains(soup: &PrimordialSoup) -> Vec<ChainId> {
    soup.molecules
        .iter()
        .filter(|m| m.is_amino_acid())
        .map(|m| m.id)
        .collect()
}

impl ChainsView {
    fn step(&mut self) {
        self.soup.simulate_step();
        self.view.sync(&self.soup.events, live_chains(&self.soup));
    }

    /// Steps the soup at a fixed rate whatever the frame rate. Animations keep
    /// running while paused so a single step plays out in full.
    fn advance(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f64().min(0.25);
        self.last_tick = now;
        self.view.advance(elapsed as f32);
        if !self.running {
            return;
        }
        self.pending += elapsed * CHAIN_STEPS_PER_SECOND;
        while self.pending >= 1.0 {
            self.step();
            self.pending -= 1.0;
        }
    }

    pub(super) fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        self.advance();
        let next = self.controls_ui(ui);
        self.rows_ui(ui);

        if self.running || self.view.is_animating() {
            ui.ctx().request_repaint();
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
        next
    }

    /// The top bar: buttons, counters and the colour legend.
    fn controls_ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let mut next = None;
        egui::Panel::top("chain_controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                if back_button(ui) {
                    next = Some(Screen::Menu);
                }
                ui.separator();
                if ui
                    .button(if self.running { "Pause" } else { "Resume" })
                    .clicked()
                {
                    self.running = !self.running;
                }
                if ui
                    .add_enabled(!self.running, egui::Button::new("Step"))
                    .clicked()
                {
                    self.step();
                }
                if ui.button("Reset").clicked() {
                    *self = Self::default();
                }
                ui.separator();
                ui.label(format!("Step {}", self.soup.step));
                ui.label(format!("Chains {}", self.view.rows().len()));
            });
            ui.horizontal_wrapped(legend_ui);
        });
        next
    }

    fn rows_ui(&self, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(MENU_BG).inner_margin(16.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for row in self.view.rows() {
                            draw_row(ui, row);
                        }
                    });
            });
    }
}
