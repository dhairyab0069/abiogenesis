//! The GUI: a launcher menu, the molecule simulation, the chain build-up view and the genes-and-proteins scene.

mod chains_view;
mod formation_view;
mod fps;
mod molecule_draw;
mod soup_view;
mod sparkline;
mod tiles;

pub use chains_view::ChainsView;
pub use formation_view::FormationView;
pub use soup_view::SoupView;

use eframe::egui::{self, Color32, RichText, Vec2};

/// Background of the menu and of the chains screen.
const MENU_BG: Color32 = Color32::from_rgb(0x18, 0x1c, 0x1f);

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
    Formation,
}

impl App {
    pub fn new(start: Start) -> Self {
        let screen = match start {
            Start::Menu => Screen::Menu,
            Start::Molecules => Screen::Soup(Box::default()),
            Start::Chains => Screen::Chains(Box::default()),
            Start::Formation => Screen::Formation(Box::default()),
        };
        Self { screen }
    }
}

#[derive(Default)]
enum Screen {
    #[default]
    Menu,
    Soup(Box<SoupView>),
    Chains(Box<ChainsView>),
    Formation(Box<FormationView>),
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let next = match &mut self.screen {
            Screen::Menu => menu(ui),
            Screen::Soup(view) => view.ui(ui),
            Screen::Chains(view) => view.ui(ui),
            Screen::Formation(view) => view.ui(ui),
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
                next = Some(Screen::Chains(Box::default()));
            }
            ui.add_space(12.0);
            if ui.add_sized(size, egui::Button::new("Genes and Proteins")).clicked() {
                next = Some(Screen::Formation(Box::default()));
            }
        });
    });
    next
}
