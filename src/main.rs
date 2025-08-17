#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod ui_helpers;
mod wad;

use app::App;
use arboard::Clipboard;
use config::Config;

fn main() {
    let config: Config = confy::load("wadlauncher", None).unwrap();
    let width = config.window_width.unwrap_or(640.0);
    let height = config.window_height.unwrap_or(480.0);
    eframe::run_native(
        "wadlauncher",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([width, height]),
            ..Default::default()
        },
        Box::new(|_| {
            Ok(Box::new(App {
                config,
                clipboard: Clipboard::new().unwrap(),
                titlepic_texture: None,
                last_iwad_path: None,
                last_wad_path: None,
            }))
        }),
    )
    .unwrap();
}
