#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use regex::bytes::Regex;
use serde_derive::{Deserialize, Serialize};
use std::{path::Path, process::Command};
use wad::load_wad_file;
use window_titles::ConnectionTrait;

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
        "Doom Discord RPC Launcher",
        eframe::NativeOptions {
            drag_and_drop_support: true,
            initial_window_size: Some(egui::vec2(640.0, 480.0)),
            ..Default::default()
        },
        Box::new(|_| Box::new(App { config })),
    )
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
                    let mut dialog = rfd::FileDialog::new().add_filter("*.exe", &["exe"]);
                    if let Some(path) = cfg.engine_path.as_ref() {
                        dialog = dialog.set_directory(Path::new(path).parent().unwrap());
                    }
                    let dialog_result = dialog.pick_file();
                    if let Some(path) = dialog_result {
                        cfg.engine_path = Some(path.display().to_string());
                        confy::store("drpcl", None, &cfg).unwrap();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("IWAD:");
                ui.monospace(cfg.iwad_path.as_ref().unwrap_or(&"<Empty>".to_owned()));
                if ui.button("Configure").clicked() {
                    let mut dialog = rfd::FileDialog::new().add_filter("*.WAD", &["WAD", "wad"]);
                    if let Some(path) = cfg.iwad_path.as_ref() {
                        dialog = dialog.set_directory(Path::new(path).parent().unwrap());
                    }
                    let dialog_result = dialog.pick_file();
                    if let Some(path) = dialog_result {
                        cfg.iwad_path = Some(path.display().to_string());
                        confy::store("drpcl", None, &cfg).unwrap();
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Input files:");
                if ui.button("Add").clicked() {
                    let mut dialog = rfd::FileDialog::new()
                        .add_filter("*.WAD", &["WAD", "wad"])
                        .add_filter("*.deh", &["deh"])
                        .add_filter("*.pk3", &["pk3"]);
                    if let Some(path) = cfg.iwad_path.as_ref() {
                        dialog = dialog.set_directory(Path::new(path).parent().unwrap());
                    }
                    let dialog_result = dialog.pick_file();
                    if let Some(path) = dialog_result {
                        cfg.input_paths
                            .as_mut()
                            .unwrap()
                            .push(path.display().to_string());
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

                let first_wadfile_path = &cfg.input_paths.as_ref().unwrap()[0];
                let title = get_wad_title(first_wadfile_path).unwrap_or_else(|| {
                    let file_name = Path::new(first_wadfile_path.as_str()).file_stem().unwrap();
                    let result = file_name.to_str().unwrap().to_owned();
                    println!("Using filename as a title instead: {}", result);
                    result
                });

                println!("Launching game:\n{:?}\n", cmd);
                cmd.spawn().unwrap();

                std::thread::sleep(std::time::Duration::from_secs(5));

                let connection = window_titles::Connection::new().unwrap();

                std::thread::spawn({
                    let mut drpc_client = discord_rpc_client::Client::new(1051907896243400746);
                    drpc_client.start();

                    move || loop {
                        let windows: Vec<String> = connection.window_titles().unwrap();
                        dbg!(&windows);

                        let window: String = windows
                            .into_iter()
                            .filter(|s| s.contains("DOOM 2: Hell on Earth"))
                            .collect();

                        let parts: Vec<&str> = window.split(" - ").collect();

                        if parts.len() > 1 {
                            let level = parts[0];
                            let game = parts[1];

                            dbg!((level, game));

                            if let Err(why) = drpc_client.set_activity(|a| {
                                println!("Setting discord presence...");
                                a.details(format!("{} - {}", title, level))
                                    .assets(|ass| ass.large_image("doom_ii").large_text("Doom 2"))
                            }) {
                                println!("Failed to set presence: {}", why);
                            }
                        }

                        std::thread::sleep(std::time::Duration::from_secs(5));
                        println!();
                    }
                });
            }
        });

        input_path_indexes_to_remove.sort();

        for index in input_path_indexes_to_remove.iter().rev() {
            cfg.input_paths.as_mut().unwrap().remove(*index);
            confy::store("drpcl", None, &cfg).unwrap();
        }
    }
}

fn get_wad_title(path: &String) -> Option<String> {
    let re = Regex::new(r"Title\s+:(.+)").unwrap();

    println!("get_wad_title: Loading WAD file: {}", path);
    let wad = load_wad_file(path).expect("get_wad_title: Couldn't load the WAD!");

    let wadinfo = wad
        .entry_iter()
        .find(|entry| entry.id.display() == "WADINFO");

    if let Some(entry) = wadinfo {
        let title = re.captures_iter(entry.lump).next().map(|capture| {
            let first_capture_bytes = capture.get(1).unwrap().as_bytes();
            let first_capture_str = std::str::from_utf8(first_capture_bytes).unwrap();
            first_capture_str.trim().to_owned()
        });

        if title.is_none() {
            println!("get_wad_title: Couldn't find the WAD title!");
        }

        title
    } else {
        println!("get_wad_title: Couldn't find the WADINFO lump!");
        None
    }
}
