#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::process::Command;
use tinyfiledialogs as tfd;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct Config {
    engine_path: Option<String>,
    iwad_path: Option<String>,
    input_paths: Option<Vec<String>>,
}

struct App {
    config: Config,
}

fn main() {
    let mut config: Config = confy::load("drpcl", None).unwrap();
    config.input_paths.get_or_insert_with(|| Vec::new());

    eframe::run_native(
        "wadlauncher",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(App { config }))),
    )
    .unwrap();
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let cfg = &mut self.config;
        let mut input_path_indexes_to_remove = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Game engine:");
                ui.monospace(cfg.engine_path.as_ref().unwrap_or(&"<Empty>".to_owned()));
                if ui.button("Configure").clicked() {
                    let path = tfd::open_file_dialog(
                        "Select Game Engine",
                        cfg.engine_path.as_ref().map(|p| p.as_str()).unwrap_or("."),
                        None,
                    );
                    if let Some(path) = path {
                        cfg.engine_path = Some(path);
                        confy::store("drpcl", None, &cfg).unwrap();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("IWAD:");
                ui.monospace(cfg.iwad_path.as_ref().unwrap_or(&"<Empty>".to_owned()));
                if ui.button("Configure").clicked() {
                    let path = tfd::open_file_dialog(
                        "Select IWAD",
                        cfg.iwad_path.as_ref().map(|p| p.as_str()).unwrap_or("."),
                        Some((&["*.WAD", "*.wad"], "WAD files (*.WAD, *.wad)")),
                    );
                    if let Some(path) = path {
                        cfg.iwad_path = Some(path);
                        confy::store("drpcl", None, &cfg).unwrap();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Input files:");
                if ui.button("Add").clicked() {
                    let path = tfd::open_file_dialog(
                        "Add Input File",
                        cfg.iwad_path.as_ref().map(|p| p.as_str()).unwrap_or("."),
                        Some((
                            &["*.WAD", "*.wad", "*.deh", "*.pk3"],
                            "Supported files (*.WAD, *.wad, *.deh, *.pk3)",
                        )),
                    );
                    if let Some(path) = path {
                        cfg.input_paths.as_mut().unwrap().push(path);
                        confy::store("drpcl", None, &cfg).unwrap();
                    }
                }
            });

            ui.group(|ui| {
                if cfg.input_paths.as_ref().unwrap().is_empty() {
                    ui.label("<Empty>");
                }

                for (index, path) in cfg.input_paths.as_ref().unwrap().iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(path.clone());
                        if ui.button("Remove").clicked() {
                            input_path_indexes_to_remove.push(index);
                        }
                    });
                }
            });

            if ui.button("Launch").clicked() {
                let mut cmd = Command::new(cfg.engine_path.as_ref().unwrap());

                cmd.args(cfg.input_paths.as_ref().unwrap())
                    .arg("-iwad")
                    .arg(cfg.iwad_path.as_ref().unwrap());

                println!("Launching game:\n{:?}\n", cmd);
                cmd.spawn().unwrap();
            }
        });

        input_path_indexes_to_remove.sort();

        for index in input_path_indexes_to_remove.iter().rev() {
            cfg.input_paths.as_mut().unwrap().remove(*index);
            confy::store("drpcl", None, &cfg).unwrap();
        }
    }
}
