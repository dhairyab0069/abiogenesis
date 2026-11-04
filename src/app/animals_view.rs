//! The "Animals: from sea to land" screen. Left: the simulated lineage (a shoreline scene, the
//! leading animal's portrait, a scrubber). Right: the real-history story in short chapters that follow
//! along as the simulation reaches matching milestones. Nothing here decides what happens: the run is
//! computed by `crate::animals`, the shapes come from `crate::animalviz`.

use super::animal_draw::{draw_portrait, draw_scene};
use super::{MENU_BG, Screen, back_button};
use crate::animals::{
    CHAPTERS, DEFAULT_SEED, Environment, GENERATIONS, Milestone, Organ, Run, Summary, Zone,
    simulation_note,
};
use crate::animalviz::Shore;
use eframe::egui::{
    self, Align, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, vec2,
};
use std::time::{Duration, Instant};

const MIN_SPEED: f32 = 5.0;
const MAX_SPEED: f32 = 240.0;
const NOTE: &str = "A small, honest simplification, not a biology textbook: every organ is one number, the shore is four zones, \
and nothing here is a measurement.";
const STORY_NOTE: &str = "The story is told in words, from what scientists broadly agree on. It is a story, not data: \
dates are rough, and the real history is messier and still being worked out.";

pub struct AnimalsView {
    run: Run,
    /// The default world with the same seed, for the what-if comparison.
    baseline: Summary,
    seed: u64,
    /// The world being tried in the what-if window (applied with "Try this world").
    pending: Environment,
    generation: f32,
    playing: bool,
    speed: f32,
    clock: f32,
    side_by_side: bool,
    what_if_open: bool,
    /// A chapter the reader opened by hand.
    pinned: Option<usize>,
    last_current: usize,
    last_tick: Instant,
}

impl Default for AnimalsView {
    fn default() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }
}

impl AnimalsView {
    fn with_seed(seed: u64) -> Self {
        let run = Run::new(Environment::default(), seed, GENERATIONS);
        let baseline = run.summary();
        Self {
            run,
            baseline,
            seed,
            pending: Environment::default(),
            generation: 0.0,
            playing: true,
            speed: 40.0,
            clock: 0.0,
            side_by_side: true,
            what_if_open: false,
            pinned: None,
            last_current: 0,
            last_tick: Instant::now(),
        }
    }

    fn restart_with(&mut self, env: Environment, seed: u64) {
        if seed != self.seed {
            self.baseline = Run::new(Environment::default(), seed, GENERATIONS).summary();
        }
        self.run = Run::new(env, seed, GENERATIONS);
        self.seed = seed;
        self.generation = 0.0;
        self.playing = true;
        self.pinned = None;
        self.last_current = 0;
    }

    fn last_generation(&self) -> f32 {
        self.run.last().generation as f32
    }

    fn at_end(&self) -> bool {
        self.generation >= self.last_generation()
    }

    pub(super) fn ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32().min(0.25);
        self.last_tick = now;
        if self.playing {
            self.clock += dt;
            self.generation = (self.generation + dt * self.speed).min(self.last_generation());
            if self.at_end() {
                self.playing = false;
            }
        }
        let now_gen = self.generation.floor() as u32;

        let next = self.controls_ui(ui);
        if self.side_by_side {
            egui::Panel::right("animal_story")
                .resizable(true)
                .default_size(380.0)
                .size_range(300.0..=560.0)
                .show(ui, |ui| self.story_ui(ui, now_gen));
        }
        egui::Panel::left("animal_lineage")
            .resizable(false)
            .exact_size(250.0)
            .show(ui, |ui| self.lineage_ui(ui, now_gen));
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(MENU_BG))
            .show(ui, |ui| self.scene_ui(ui, now_gen));
        self.what_if_ui(ui.ctx());

        if self.playing {
            ui.ctx().request_repaint();
        } else {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
        next
    }

    fn controls_ui(&mut self, ui: &mut egui::Ui) -> Option<Screen> {
        let mut next = None;
        egui::Panel::top("animal_controls").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if back_button(ui) {
                    next = Some(Screen::Menu);
                }
                ui.separator();
                let label = if self.playing {
                    "Pause"
                } else if self.at_end() {
                    "Replay"
                } else {
                    "Play"
                };
                if ui.button(label).clicked() {
                    if self.at_end() {
                        self.generation = 0.0;
                        self.pinned = None;
                    }
                    self.playing = !self.playing || self.generation == 0.0;
                }
                ui.add(
                    egui::Slider::new(&mut self.speed, MIN_SPEED..=MAX_SPEED)
                        .logarithmic(true)
                        .text("Generations per second"),
                );
                ui.separator();
                ui.selectable_value(
                    &mut self.side_by_side,
                    true,
                    "Side by side with the real story",
                );
                ui.selectable_value(&mut self.side_by_side, false, "Simulation only");
                ui.separator();
                if ui.selectable_label(self.what_if_open, "What if?").clicked() {
                    self.what_if_open = !self.what_if_open;
                }
                if ui
                    .button("Copy run as CSV")
                    .on_hover_text("Copies one row per generation to the clipboard")
                    .clicked()
                {
                    ui.ctx().copy_text(self.run.to_csv());
                }
            });
            self.scrubber_ui(ui);
        });
        next
    }

    /// A timeline: drag or click to move through the generations; chapter markers on it.
    fn scrubber_ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click_and_drag());
        let last = self.last_generation().max(1.0);
        if response.is_pointer_button_down_on()
            && let Some(p) = response.interact_pointer_pos()
        {
            self.generation = generation_at(rect, p.x, last);
            self.playing = false;
        }
        let painter = ui.painter_at(rect);
        let track_y = rect.top() + 11.0;
        let (x0, x1) = (rect.left() + 8.0, rect.right() - 8.0);
        painter.line_segment(
            [Pos2::new(x0, track_y), Pos2::new(x1, track_y)],
            Stroke::new(3.0, ui.visuals().widgets.inactive.bg_fill),
        );
        let head_x = x0 + (x1 - x0) * (self.generation / last);
        painter.line_segment(
            [Pos2::new(x0, track_y), Pos2::new(head_x, track_y)],
            Stroke::new(3.0, Color32::from_rgb(90, 170, 200)),
        );
        let now_gen = self.generation.floor() as u32;
        let hover = response.hover_pos();
        let mut hovered_tip = None;
        for (m, g) in &self.run.milestones {
            let x = x0 + (x1 - x0) * (*g as f32 / last);
            let done = *g <= now_gen;
            let color = if done {
                Color32::from_rgb(250, 210, 110)
            } else {
                Color32::from_white_alpha(70)
            };
            painter.circle_filled(Pos2::new(x, track_y), 4.0, color);
            if hover.is_some_and(|h| (h.x - x).abs() < 6.0 && (h.y - track_y).abs() < 10.0) {
                hovered_tip = Some(format!("{}: generation {g}", m.label()));
            }
        }
        painter.circle_filled(Pos2::new(head_x, track_y), 6.0, Color32::WHITE);
        painter.text(
            Pos2::new(rect.left() + 8.0, rect.bottom() - 1.0),
            Align2::LEFT_BOTTOM,
            format!("generation {now_gen} of {}", self.run.last().generation),
            FontId::proportional(11.0),
            ui.visuals().weak_text_color(),
        );
        let ms = self.run.milestones_by(now_gen);
        if let Some((m, _)) = ms.last() {
            painter.text(
                Pos2::new(rect.right() - 8.0, rect.bottom() - 1.0),
                Align2::RIGHT_BOTTOM,
                format!("latest: {}", m.label()),
                FontId::proportional(11.0),
                ui.visuals().weak_text_color(),
            );
        }
        if let Some(tip) = hovered_tip {
            response.on_hover_text(tip);
        } else if let Some(h) = hover {
            response.on_hover_text(format!(
                "generation {}",
                generation_at(rect, h.x, last).floor()
            ));
        }
    }

    /// Left column: the leading animal and its organs.
    fn lineage_ui(&mut self, ui: &mut egui::Ui, now_gen: u32) {
        let frame = self.run.frame(now_gen);
        ui.add_space(4.0);
        ui.strong("The simulated lineage");
        ui.label(
            RichText::new(format!(
                "Leading animals live in the {}.",
                frame.frontier_zone.label().to_lowercase()
            ))
            .small(),
        );
        let (response, painter) =
            ui.allocate_painter(vec2(ui.available_width(), 160.0), Sense::hover());
        draw_portrait(
            &painter,
            response.rect,
            &frame.frontier.typical(),
            frame.frontier_zone,
            self.clock,
        );
        ui.add_space(2.0);
        let plan = if frame.frontier.bilateral >= 0.5 {
            "bilateral (a head and a tail)"
        } else {
            "radial (round, no front)"
        };
        ui.label(format!("Body plan: {plan}"));
        egui::Grid::new("animal_organs")
            .num_columns(2)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                for organ in Organ::ALL {
                    let level = frame.frontier.get(organ);
                    ui.label(RichText::new(organ.label()).small());
                    ui.add(
                        egui::ProgressBar::new(level)
                            .desired_width(110.0)
                            .desired_height(8.0),
                    );
                    ui.end_row();
                }
            });
        ui.add_space(6.0);
        if let Some(g) = self.run.extinct_at.filter(|g| *g <= now_gen) {
            ui.colored_label(
                Color32::from_rgb(230, 120, 100),
                format!("The whole lineage died out at generation {g} in this world."),
            );
        }
        ui.add(egui::Label::new(RichText::new(NOTE).weak().small()).wrap());
    }

    fn scene_ui(&mut self, ui: &mut egui::Ui, now_gen: u32) {
        let frame = self.run.frame(now_gen);
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let rect = response.rect;
        let shore = Shore::new(&self.run.env);
        let boxes = draw_scene(&painter, rect, &shore, frame, self.clock);
        if let Some(h) = response.hover_pos()
            && let Some((_, index)) = boxes.iter().rev().find(|(r, _)| r.contains(h))
        {
            let a = &frame.samples[*index];
            let text = animal_tooltip(a.zone, &a.traits.levels, a.traits.bilateral);
            response.on_hover_text(text);
        }
        let caption =
            "Each animal drawn is a sample of the living lineage. Hover one to see its organs.";
        painter.text(
            rect.left_top() + vec2(10.0, 8.0),
            Align2::LEFT_TOP,
            caption,
            FontId::proportional(11.5),
            Color32::from_white_alpha(190),
        );
    }

    /// The right column: the real story, one chapter per milestone.
    fn story_ui(&mut self, ui: &mut egui::Ui, now_gen: u32) {
        let reached = self.run.milestones_by(now_gen);
        let current = current_chapter(&reached);
        let changed = current != self.last_current;
        self.last_current = current;
        ui.add_space(4.0);
        ui.strong("The real story, roughly");
        ui.add(egui::Label::new(RichText::new(STORY_NOTE).weak().small()).wrap());
        ui.label(format!("Chapter {} of {}", current + 1, CHAPTERS.len()));
        ui.separator();
        let mut jump = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (i, chapter) in CHAPTERS.iter().enumerate() {
                let found = chapter.milestone.and_then(|m| self.run.milestones.iter().find(|(x, _)| *x == m).map(|(_, g)| *g));
                let is_reached = chapter.milestone.is_none() || found.is_some_and(|g| g <= now_gen);
                let open = i == current || self.pinned == Some(i);
                let stroke = if i == current { Stroke::new(1.5, Color32::from_rgb(250, 210, 110)) } else { ui.visuals().widgets.noninteractive.bg_stroke };
                egui::Frame::group(ui.style()).stroke(stroke).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    let header = ui.selectable_label(open, format!("{}. {}", i + 1, chapter.title));
                    if header.clicked() {
                        self.pinned = if self.pinned == Some(i) { None } else { Some(i) };
                    }
                    if changed && i == current {
                        header.scroll_to_me(Some(Align::Center));
                    }
                    let status = match (chapter.milestone, is_reached) {
                        (None, _) => "the starting point".to_string(),
                        (Some(_), true) => format!("reached at generation {}", found.unwrap_or(0)),
                        (Some(_), false) => "not reached yet in this run".to_string(),
                    };
                    ui.label(RichText::new(status).small().weak());
                    if open {
                        for (name, text) in [("Setting", chapter.setting), ("Problem", chapter.problem), ("Adaptation", chapter.adaptation), ("Result", chapter.result)] {
                            ui.add(egui::Label::new(RichText::new(name).small().color(Color32::from_rgb(250, 210, 110))));
                            ui.add(egui::Label::new(text).wrap());
                        }
                        ui.add_space(2.0);
                        let sim = match chapter.milestone {
                            Some(m) => simulation_note(m, &reached),
                            None => "The lineage starts here, from a colony with feeder, soma and germ cells.".to_string(),
                        };
                        ui.add(egui::Label::new(RichText::new("In the simulation").small().color(Color32::from_rgb(120, 190, 220))));
                        ui.add(egui::Label::new(sim).wrap());
                        ui.add(egui::Label::new(RichText::new(chapter.model).small().weak()).wrap());
                        if let Some(g) = found
                            && ui.small_button(format!("Show generation {g}")).clicked()
                        {
                            jump = Some(g);
                        }
                    } else if is_reached {
                        ui.add(egui::Label::new(RichText::new(chapter.result).small()).wrap());
                    }
                });
            }
        });
        if let Some(g) = jump {
            self.generation = g as f32;
            self.playing = false;
        }
    }

    /// The what-if window: change the world, rerun the lineage, compare with the default world.
    fn what_if_ui(&mut self, ctx: &egui::Context) {
        if !self.what_if_open {
            return;
        }
        let mut open = true;
        let mut apply = false;
        let mut reset = false;
        let mut reseed = false;
        egui::Window::new("What if? (hypothetical)").open(&mut open).anchor(Align2::RIGHT_BOTTOM, vec2(-12.0, -12.0)).resizable(false).show(ctx, |ui| {
            ui.add(egui::Label::new(RichText::new("Change the world and run the same lineage again. These are made-up worlds, not predictions, and they are one run each: chance matters.").small()).wrap());
            ui.add(egui::Slider::new(&mut self.pending.oxygen, Environment::MIN_OXYGEN..=1.0).text("Oxygen in the sea"));
            ui.add(egui::Slider::new(&mut self.pending.slope, 0.0..=1.0).text("Shore steepness"));
            ui.add(egui::Slider::new(&mut self.pending.predators, 0.0..=1.0).text("Predator pressure"));
            ui.horizontal(|ui| {
                apply = ui.button("Try this world").clicked();
                reset = ui.button("Back to the default world").clicked();
                reseed = ui.button("New seed").clicked();
            });
            ui.separator();
            let there = self.run.summary();
            ui.label(format!("Seed {}. This world, after {} generations:", self.seed, there.generations));
            for line in compare_lines(&self.baseline, &there) {
                ui.label(line);
            }
            egui::Grid::new("what_if_organs").num_columns(4).striped(true).show(ui, |ui| {
                ui.label(RichText::new("Leading animals").small().weak());
                ui.label(RichText::new("default").small().weak());
                ui.label(RichText::new("this world").small().weak());
                ui.label("");
                ui.end_row();
                for (name, a, b) in organ_rows(&self.baseline, &there) {
                    ui.label(RichText::new(name).small());
                    ui.label(format!("{:.0}%", a * 100.0));
                    ui.label(format!("{:.0}%", b * 100.0));
                    ui.label(RichText::new(change_arrow(a, b)).small());
                    ui.end_row();
                }
            });
            ui.add(egui::Label::new(RichText::new("This is a first taste of the scenario lab to come: whole scenarios, with forced as well as natural selection.").small().weak()).wrap());
        });
        if !open {
            self.what_if_open = false;
        }
        if reset {
            self.pending = Environment::default();
        }
        if apply || reset {
            let env = self.pending;
            self.restart_with(env, self.seed);
        }
        if reseed {
            let seed = rand::random::<u64>() % 1_000_000;
            let env = self.pending;
            self.restart_with(env, seed);
        }
    }
}

/// Which chapter is current: the one for the latest milestone reached, or the prologue.
pub(super) fn current_chapter(reached: &[(Milestone, u32)]) -> usize {
    reached
        .iter()
        .max_by_key(|(m, g)| (*g, m.story_rank()))
        .map(|(m, _)| m.story_rank() + 1)
        .unwrap_or(0)
}

/// The generation under screen x on a scrubber over `rect` for a run of `last` generations.
pub(super) fn generation_at(rect: Rect, x: f32, last: f32) -> f32 {
    let (x0, x1) = (rect.left() + 8.0, rect.right() - 8.0);
    (((x - x0) / (x1 - x0).max(1.0)).clamp(0.0, 1.0) * last).round()
}

/// Leading animals' organ levels in the default world and in this one, one row per organ.
pub(super) fn organ_rows(base: &Summary, other: &Summary) -> Vec<(&'static str, f32, f32)> {
    Organ::ALL
        .iter()
        .map(|o| (o.label(), base.frontier.get(*o), other.frontier.get(*o)))
        .collect()
}

fn change_arrow(a: f32, b: f32) -> &'static str {
    if b - a > 0.1 {
        "more"
    } else if a - b > 0.1 {
        "less"
    } else {
        "about the same"
    }
}

/// A few lines comparing two worlds' outcomes.
pub(super) fn compare_lines(base: &Summary, other: &Summary) -> Vec<String> {
    let mut lines = Vec::new();
    let reached = |s: &Summary| {
        format!(
            "{} of {} milestones",
            s.milestones.len(),
            Milestone::ALL.len()
        )
    };
    lines.push(format!(
        "Default world: {}, leading animals in the {}, {} on land.",
        reached(base),
        base.frontier_zone.label().to_lowercase(),
        base.counts[Zone::Land.index()]
    ));
    match other.extinct_at {
        Some(g) => lines.push(format!(
            "This world: the lineage died out at generation {g}."
        )),
        None => lines.push(format!(
            "This world: {}, leading animals in the {}, {} on land.",
            reached(other),
            other.frontier_zone.label().to_lowercase(),
            other.counts[Zone::Land.index()]
        )),
    }
    let first_land = |s: &Summary| {
        s.milestones
            .iter()
            .find(|(m, _)| *m == Milestone::FirstLandAnimals)
            .map(|(_, g)| *g)
    };
    match (first_land(base), first_land(other)) {
        (Some(a), Some(b)) if b > a => lines.push(format!(
            "First land animals came {} generations later than in the default world.",
            b - a
        )),
        (Some(a), Some(b)) if b < a => lines.push(format!(
            "First land animals came {} generations sooner than in the default world.",
            a - b
        )),
        (Some(_), Some(_)) => {
            lines.push("First land animals came at the same generation.".to_string())
        }
        (Some(_), None) => lines.push("No land animals in this world (in this run).".to_string()),
        (None, Some(_)) => {
            lines.push("Land animals appeared here but not in the default run.".to_string())
        }
        (None, None) => {}
    }
    lines
}

/// Hover text for one drawn animal.
pub(super) fn animal_tooltip(zone: Zone, levels: &[f32], bilateral: bool) -> String {
    let mut text = format!(
        "{}: {} body",
        zone.label(),
        if bilateral { "bilateral" } else { "radial" }
    );
    for (organ, level) in Organ::ALL.iter().zip(levels) {
        if *level >= 0.1 {
            text.push_str(&format!("\n{} {:.0}%", organ.label(), level * 100.0));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_chapter_follows_the_latest_milestone() {
        assert_eq!(current_chapter(&[]), 0, "the prologue first");
        let reached = [
            (Milestone::SymmetryAndGut, 10),
            (Milestone::Lungs, 30),
            (Milestone::Backbone, 20),
        ];
        assert_eq!(current_chapter(&reached), Milestone::Lungs.story_rank() + 1);
        assert_eq!(current_chapter(&reached[..1]), 1);
    }

    #[test]
    fn the_scrubber_maps_x_to_generations_and_clamps() {
        let rect = Rect::from_min_max(Pos2::new(10.0, 0.0), Pos2::new(210.0, 20.0));
        assert_eq!(generation_at(rect, 0.0, 1000.0), 0.0);
        assert_eq!(generation_at(rect, 500.0, 1000.0), 1000.0);
        assert_eq!(generation_at(rect, 110.0, 1000.0), 500.0);
    }

    #[test]
    fn the_comparison_names_the_differences() {
        let base = Run::new(Environment::default(), DEFAULT_SEED, GENERATIONS).summary();
        let same = compare_lines(&base, &base);
        assert!(same.iter().any(|l| l.contains("at the same generation")));
        let dead = Summary {
            extinct_at: Some(42),
            ..base.clone()
        };
        assert!(
            compare_lines(&base, &dead)
                .iter()
                .any(|l| l.contains("died out at generation 42"))
        );
        let none = Summary {
            milestones: Vec::new(),
            ..base.clone()
        };
        assert!(
            compare_lines(&base, &none)
                .iter()
                .any(|l| l.contains("No land animals"))
        );
        let later = Summary {
            milestones: base.milestones.iter().map(|(m, g)| (*m, g + 50)).collect(),
            ..base.clone()
        };
        assert!(
            compare_lines(&base, &later)
                .iter()
                .any(|l| l.contains("50 generations later"))
        );
        assert_eq!(organ_rows(&base, &base).len(), crate::animals::ORGANS);
        assert!(organ_rows(&base, &base).iter().all(|(_, a, b)| a == b));
    }

    #[test]
    fn the_tooltip_lists_only_organs_that_are_there() {
        let mut levels = [0.0; crate::animals::ORGANS];
        levels[Organ::Gills.index()] = 0.8;
        levels[Organ::Eyes.index()] = 0.05;
        let text = animal_tooltip(Zone::Shallows, &levels, true);
        assert!(
            text.contains("Shallows") && text.contains("bilateral") && text.contains("gills 80%")
        );
        assert!(!text.contains("eyes"));
    }

    fn render(view: &mut AnimalsView, frames: usize) {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1100.0, 700.0))),
            ..Default::default()
        };
        for _ in 0..frames {
            let mut output = ctx.run_ui(input.clone(), |ui| {
                view.ui(ui);
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn every_stage_of_the_run_draws_in_both_layouts_without_panicking() {
        let mut view = AnimalsView {
            what_if_open: true,
            ..AnimalsView::default()
        };
        for generation in [0.0, 150.0, 400.0, 800.0, 1200.0, 1600.0] {
            for side_by_side in [true, false] {
                view.generation = generation;
                view.side_by_side = side_by_side;
                view.playing = false;
                render(&mut view, 2);
            }
        }
        // a pinned chapter and the extreme worlds draw too
        view.pinned = Some(3);
        view.restart_with(
            Environment {
                oxygen: 0.35,
                slope: 1.0,
                predators: 1.0,
            },
            3,
        );
        for generation in [0.0, 800.0, 1600.0] {
            view.generation = generation;
            view.playing = false;
            render(&mut view, 1);
        }
    }

    #[test]
    fn a_new_view_starts_playing_from_generation_zero_with_the_story_beside_it() {
        let v = AnimalsView::default();
        assert_eq!((v.generation, v.playing, v.side_by_side), (0.0, true, true));
        assert_eq!(v.run.env, Environment::default());
        assert_eq!(v.baseline, v.run.summary());
    }

    #[test]
    fn trying_a_world_restarts_the_run_and_keeps_the_default_baseline() {
        let mut v = AnimalsView::default();
        let base = v.baseline.clone();
        v.generation = 500.0;
        v.restart_with(
            Environment {
                oxygen: 0.4,
                ..Environment::default()
            },
            v.seed,
        );
        assert_eq!(v.generation, 0.0);
        assert_eq!(v.run.env.oxygen, 0.4);
        assert_eq!(v.baseline, base, "the baseline is still the default world");
        v.restart_with(Environment::default(), 77);
        assert_eq!(v.seed, 77);
        assert_eq!(
            v.baseline,
            v.run.summary(),
            "a new seed recomputes the baseline"
        );
    }
}
