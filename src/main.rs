#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use serde_derive::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tinyfiledialogs as tfd;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    pub engine_path: Option<String>,
    pub iwad_path: Option<String>,
    pub input_paths: Vec<String>,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            engine_path: None,
            iwad_path: None,
            input_paths: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tabs: Vec<TabConfig>,
    pub selected_tab: usize,
    pub last_engine_dir: Option<String>,
    pub last_iwad_dir: Option<String>,
    pub last_input_dir: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tabs: vec![TabConfig::default()],
            selected_tab: 0,
            last_engine_dir: None,
            last_iwad_dir: None,
            last_input_dir: None,
        }
    }
}

struct App {
    config: Config,
}

fn main() {
    let config: Config = confy::load("wadlauncher", None).unwrap();
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

fn sanitize_tab_name_part(s: &str) -> String {
    let mut out = s.to_string();
    // Replace dsda-doom (case-insensitive) with dsda
    out = regex::RegexBuilder::new("dsda-doom")
        .case_insensitive(true)
        .build()
        .unwrap()
        .replace_all(&out, "dsda")
        .to_string();
    // Remove Linux, Windows, Win, Mac, MacOS (case-insensitive)
    out = regex::RegexBuilder::new(r"(?i)(linux|windows|win|macos|mac)")
        .case_insensitive(true)
        .build()
        .unwrap()
        .replace_all(&out, "")
        .to_string();
    // Remove leading/trailing special characters (hyphens, underscores, spaces, dots)
    out = regex::Regex::new(r"^[\s._-]+|[\s._-]+$")
        .unwrap()
        .replace_all(&out, "")
        .to_string();
    out.to_uppercase().trim().to_string()
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut store_config = false;
        let cfg = &mut self.config;

        let mut input_path_indexes_to_remove = Vec::new();

        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in cfg.tabs.iter().enumerate() {
                    let tab_title = if tab.engine_path.is_none()
                        && tab.iwad_path.is_none()
                        && tab.input_paths.is_empty()
                    {
                        "New Tab".to_owned()
                    } else {
                        let mut title = String::new();
                        if let Some(engine) = &tab.engine_path {
                            title.push_str(&sanitize_tab_name_part(
                                &std::path::Path::new(engine)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(engine),
                            ));
                        }
                        if let Some(iwad) = &tab.iwad_path {
                            if !title.is_empty() {
                                title.push_str(" : ");
                            }
                            title.push_str(&sanitize_tab_name_part(
                                &std::path::Path::new(iwad)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(iwad),
                            ));
                        }
                        if let Some(wad) = tab.input_paths.get(0) {
                            if !title.is_empty() {
                                title.push_str(" : ");
                            }
                            title.push_str(&sanitize_tab_name_part(
                                &std::path::Path::new(wad)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(wad),
                            ));
                        }
                        title
                    };
                    let selected = i == cfg.selected_tab;
                    if ui.selectable_label(selected, tab_title).clicked() {
                        cfg.selected_tab = i;
                        store_config = true;
                    }
                    if cfg.tabs.len() > 1 {
                        if ui.button("×").on_hover_text("Close tab").clicked() {
                            println!("closing tab {}", i);
                            cfg.tabs.remove(i);
                            if cfg.selected_tab >= cfg.tabs.len() {
                                cfg.selected_tab = cfg.tabs.len() - 1;
                            }
                            store_config = true;
                            break;
                        }
                    }
                }
                if ui.button("+").on_hover_text("New tab").clicked() {
                    cfg.tabs.push(TabConfig::default());
                    cfg.selected_tab = cfg.tabs.len() - 1;
                    store_config = true;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();

            ui.horizontal(|ui| {
                ui.label("Game engine:");
                ui.monospace(
                    tab_config
                        .engine_path
                        .as_ref()
                        .unwrap_or(&"<Empty>".to_owned()),
                );
                if ui.button("Configure").clicked() {
                    let start_dir = tab_config
                        .engine_path
                        .as_ref()
                        .and_then(|p| Path::new(p).parent().map(|d| d.to_str().unwrap_or(".")))
                        .or(cfg.last_engine_dir.as_deref())
                        .unwrap_or(".");
                    let path = tfd::open_file_dialog("Select Game Engine", start_dir, None);
                    if let Some(path) = path {
                        tab_config.engine_path = Some(path.clone());
                        cfg.last_engine_dir = Path::new(&path)
                            .parent()
                            .map(|d| d.to_string_lossy().to_string());
                        store_config = true;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("IWAD:");
                ui.monospace(
                    tab_config
                        .iwad_path
                        .as_ref()
                        .unwrap_or(&"<Empty>".to_owned()),
                );
                if ui.button("Configure").clicked() {
                    let start_dir = tab_config
                        .iwad_path
                        .as_ref()
                        .and_then(|p| Path::new(p).parent().map(|d| d.to_str().unwrap_or(".")))
                        .or(cfg.last_iwad_dir.as_deref())
                        .unwrap_or(".");
                    let path = tfd::open_file_dialog(
                        "Select IWAD",
                        start_dir,
                        Some((&["*.WAD", "*.wad"], "WAD files (*.WAD, *.wad)")),
                    );
                    if let Some(path) = path {
                        tab_config.iwad_path = Some(path.clone());
                        cfg.last_iwad_dir = Path::new(&path)
                            .parent()
                            .map(|d| d.to_string_lossy().to_string());
                        store_config = true;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Input files:");
                if ui.button("Add").clicked() {
                    let start_dir = if let Some(last) = tab_config.input_paths.last() {
                        Path::new(last)
                            .parent()
                            .map(|d| d.to_str().unwrap_or("."))
                            .unwrap_or(".")
                    } else {
                        cfg.last_input_dir.as_deref().unwrap_or(".")
                    };
                    let path = tfd::open_file_dialog(
                        "Add Input File",
                        start_dir,
                        Some((
                            &["*.WAD", "*.wad", "*.deh", "*.pk3"],
                            "Supported files (*.WAD, *.wad, *.deh, *.pk3)",
                        )),
                    );
                    if let Some(path) = path {
                        tab_config.input_paths.push(path.clone());
                        cfg.last_input_dir = Path::new(&path)
                            .parent()
                            .map(|d| d.to_string_lossy().to_string());
                        store_config = true;
                    }
                }
            });

            ui.group(|ui| {
                if tab_config.input_paths.is_empty() {
                    ui.label("<Empty>");
                }

                for (index, path) in tab_config.input_paths.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(path.clone());
                        if ui.button("Remove").clicked() {
                            input_path_indexes_to_remove.push(index);
                        }
                    });
                }
            });

            if ui.button("Launch").clicked() {
                let mut cmd = Command::new(tab_config.engine_path.as_ref().unwrap());

                cmd.args(&tab_config.input_paths)
                    .arg("-iwad")
                    .arg(tab_config.iwad_path.as_ref().unwrap());

                println!("Launching game:\n{:?}\n", cmd);
                cmd.spawn().unwrap();
            }
        });

        let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();
        input_path_indexes_to_remove.sort();

        for index in input_path_indexes_to_remove.iter().rev() {
            tab_config.input_paths.remove(*index);
            store_config = true;
        }

        if store_config {
            confy::store("wadlauncher", None, &*cfg).unwrap();
        }
    }
}
