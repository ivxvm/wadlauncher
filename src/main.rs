#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::ColorImage;
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
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tabs: vec![TabConfig::default()],
            selected_tab: 0,
            last_engine_dir: None,
            last_iwad_dir: None,
            last_input_dir: None,
            window_width: Some(640.0),
            window_height: Some(480.0),
        }
    }
}

struct App {
    config: Config,
    titlepic_texture: Option<egui::TextureHandle>,
    last_iwad_path: Option<String>,
    last_wad_path: Option<String>,
}

fn load_playpal_lump(iwad_path: Option<&str>, wad_path: Option<&str>) -> Option<Vec<u8>> {
    for path in [wad_path, iwad_path].into_iter().flatten() {
        if let Ok(wad) = wad::load_wad_file(path) {
            if let Some(data) = wad.by_id(b"PLAYPAL") {
                return Some(data[0..768].to_vec());
            }
        }
    }
    None
}

fn load_titlepic_lump(
    iwad_path: Option<&str>,
    wad_path: Option<&str>,
) -> Option<(String, Vec<u8>)> {
    const TITLE_LUMPS: [&str; 3] = ["TITLEPIC", "TITLE", "HTITLE"];
    for path in [wad_path, iwad_path].into_iter().flatten() {
        if let Ok(wad) = wad::load_wad_file(path) {
            for lump in TITLE_LUMPS {
                let entry_id = wad::EntryId::from_str(lump).unwrap();
                if let Some(data) = wad.by_id(entry_id) {
                    return Some((lump.to_string(), data.to_vec()));
                }
            }
        }
    }
    None
}

fn get_titlepic_dimensions(data: &[u8]) -> (usize, usize) {
    let width = u16::from_le_bytes([data[0], data[1]]) as usize;
    let height = u16::from_le_bytes([data[2], data[3]]) as usize;

    if width == 0 || height == 0 || data.len() < 4 + width * 4 {
        (320, 200)
    } else {
        (width, height)
    }
}

fn decode_titlepic(data: &[u8], palette: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    // Doom picture format: column-major, with posts
    let mut out = vec![0u8; width * height * 4];
    let mut col_offsets = vec![0u32; width];
    if data.len() < width * 4 {
        return None;
    }
    for i in 0..width {
        col_offsets[i] = u32::from_le_bytes([
            data[i * 4],
            data[i * 4 + 1],
            data[i * 4 + 2],
            data[i * 4 + 3],
        ]);
    }
    for x in 0..width {
        let mut pos = col_offsets[x] as usize;
        loop {
            if pos >= data.len() {
                break;
            }
            let y_start = data[pos] as usize;
            if y_start == 255 {
                break;
            }
            let n_pixels = data[pos + 1] as usize;
            pos += 3; // skip y_start, n_pixels, unused
            for y in y_start..(y_start + n_pixels) {
                let pal_idx = data[pos] as usize;
                let dst = (y * width + x) * 4;
                out[dst + 0] = palette[pal_idx * 3 + 0];
                out[dst + 1] = palette[pal_idx * 3 + 1];
                out[dst + 2] = palette[pal_idx * 3 + 2];
                out[dst + 3] = 16;
                pos += 1;
            }
            pos += 1; // skip unused
        }
    }
    Some(out)
}

fn decode_htitle(data: &[u8], palette: &[u8]) -> Option<Vec<u8>> {
    if data.len() != 320 * 200 {
        return None;
    }
    let mut out = vec![0u8; 320 * 200 * 4];
    for i in 0..(320 * 200) {
        let pal_idx = data[i] as usize;
        out[i * 4 + 0] = palette[pal_idx * 3 + 0];
        out[i * 4 + 1] = palette[pal_idx * 3 + 1];
        out[i * 4 + 2] = palette[pal_idx * 3 + 2];
        out[i * 4 + 3] = 16;
    }
    Some(out)
}

impl App {
    fn load_titlepic(
        &mut self,
        ctx: &egui::Context,
        iwad_path: Option<&str>,
        wad_path: Option<&str>,
    ) -> Option<()> {
        self.titlepic_texture = None;
        let palette = load_playpal_lump(iwad_path, wad_path)?;
        let (lump_name, titlepic) = load_titlepic_lump(iwad_path, wad_path)?;
        let (width, height, img) = match lump_name.as_str() {
            "TITLE" | "HTITLE" => {
                let img = decode_htitle(&titlepic, &palette)
                    .or_else(|| decode_titlepic(&titlepic, &palette, 320, 200))?;
                (320, 200, img)
            }
            "TITLEPIC" | _ => {
                let (width, height) = get_titlepic_dimensions(&titlepic);
                let img = decode_titlepic(&titlepic, &palette, width, height)?;
                (width, height, img)
            }
        };
        let color_img = ColorImage::from_rgba_unmultiplied([width, height], &img);
        self.titlepic_texture =
            Some(ctx.load_texture("titlepic", color_img, egui::TextureOptions::default()));
        Some(())
    }
}

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
                titlepic_texture: None,
                last_iwad_path: None,
                last_wad_path: None,
            }))
        }),
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut store_config = false;
        // Persist window size on resize
        let win_size = ctx.available_rect().size();
        if self.config.window_width != Some(win_size.x)
            || self.config.window_height != Some(win_size.y)
        {
            self.config.window_width = Some(win_size.x);
            self.config.window_height = Some(win_size.y);
            store_config = true;
        }

        // Precompute values to avoid borrow conflicts
        let (iwad_path, wad_path, need_titlepic, last_iwad_path, last_wad_path) = {
            let cfg = &self.config;
            let tab_config = &cfg.tabs[cfg.selected_tab];
            let iwad_path = tab_config.iwad_path.clone();
            let wad_path = tab_config.input_paths.get(0).cloned();
            let mut need_titlepic = false;
            if wad_path.is_some() || iwad_path.is_some() {
                need_titlepic = self.last_iwad_path.as_deref() != iwad_path.as_deref()
                    || self.last_wad_path.as_deref() != wad_path.as_deref()
                    || self.titlepic_texture.is_none();
            }
            let last_iwad_path = iwad_path.clone();
            let last_wad_path = wad_path.clone();
            (
                iwad_path,
                wad_path,
                need_titlepic,
                last_iwad_path,
                last_wad_path,
            )
        };

        if need_titlepic {
            self.load_titlepic(ctx, iwad_path.as_deref(), wad_path.as_deref());
            self.last_iwad_path = last_iwad_path;
            self.last_wad_path = last_wad_path;
        }

        let cfg = &mut self.config;
        let mut input_path_indexes_to_remove = Vec::new();
        let mut iwad_to_load: Option<String> = None;

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
            if let Some(tex) = &self.titlepic_texture {
                ui.painter().image(
                    tex.id(),
                    ui.max_rect(),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
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
                        iwad_to_load = Some(path);
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
