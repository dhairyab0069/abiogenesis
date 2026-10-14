// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use abiogenesis::app::{App, Start};

const USAGE: &str = "\
Abiogenesis Simulation

Usage: abiogenesis [molecules | chains]

  (none)      open the launcher menu
  molecules   open the molecule simulation directly
  chains      open the amino acid chain generator directly

Options:
  -h, --help     print this help
  -V, --version  print the version";

fn main() -> eframe::Result {
    let start = match std::env::args().nth(1).as_deref() {
        None => Start::Menu,
        Some("molecules" | "soup") => Start::Molecules,
        Some("chains") => Start::Chains,
        Some("-V" | "--version") => {
            println!("abiogenesis {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("-h" | "--help") => {
            println!("{USAGE}");
            return Ok(());
        }
        Some(other) => {
            eprintln!("abiogenesis: unknown argument '{other}'\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Abiogenesis Simulation")
            .with_inner_size([1000.0, 680.0])
            .with_min_inner_size([480.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native("Abiogenesis Simulation", options, Box::new(move |_cc| Ok(Box::new(App::new(start)))))
}
