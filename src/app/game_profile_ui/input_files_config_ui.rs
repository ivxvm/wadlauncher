use crate::config::Config;
use eframe::egui;
use std::path::Path;
use tinyfiledialogs as tfd;

const CONFIGURE_BUTTON_WIDTH: f32 = 16.0;
const PATH_COLOR_SAT: f32 = 0.5;
const PATH_COLOR_VAL: f32 = 0.6;

pub(super) fn input_files_config_ui(
    ui: &mut egui::Ui,
    cfg: &mut Config,
    input_path_indexes_to_remove: &mut Vec<usize>,
    store_config: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label("Input files:");
        if ui.button("Add").clicked() {
            let tab_config = cfg.get_active_tab();
            let start_dir = tab_config
                .last_input_dir
                .as_deref()
                .or_else(|| {
                    tab_config.input_paths.last().and_then(|last| {
                        Path::new(last).parent().map(|d| d.to_str().unwrap_or("."))
                    })
                })
                .unwrap_or(".");
            let path = tfd::open_file_dialog(
                "Add Input File",
                start_dir,
                Some((
                    &["*.WAD", "*.wad", "*.deh", "*.pk3"],
                    "Supported files (*.WAD, *.wad, *.deh, *.pk3)",
                )),
            );
            if let Some(path) = path {
                let tab_config = cfg.get_active_tab_mut();
                tab_config.input_paths.push(path.clone());
                tab_config.last_input_dir = Path::new(&path)
                    .parent()
                    .map(|d| d.to_string_lossy().to_string());
                *store_config = true;
            }
        }
    });
    ui.group(|ui| {
        if cfg.get_active_tab().input_paths.is_empty() {
            ui.label("<Empty>");
        }
        let initial_len = cfg.get_active_tab().input_paths.len();
        for index in 0..initial_len {
            let path = cfg.get_active_tab().input_paths[index].clone();
            ui.horizontal(|ui| {
                let tab_config = cfg.get_active_tab_mut();
                if ui
                    .add_enabled(index > 0, egui::Button::new("/\\"))
                    .clicked()
                {
                    tab_config.input_paths.swap(index, index - 1);
                    *store_config = true;
                }

                let last_idx = tab_config.input_paths.len().saturating_sub(1);
                if ui
                    .add_enabled(index < last_idx, egui::Button::new("\\/"))
                    .clicked()
                {
                    tab_config.input_paths.swap(index, index + 1);
                    *store_config = true;
                }

                if ui
                    .add_sized(
                        egui::vec2(CONFIGURE_BUTTON_WIDTH, ui.spacing().interact_size.y),
                        egui::Button::new("..."),
                    )
                    .clicked()
                {
                    let start_dir = Path::new(&path)
                        .parent()
                        .and_then(|d| d.to_str())
                        .unwrap_or(".");
                    let sel = tfd::open_file_dialog(
                        "Replace Input File",
                        start_dir,
                        Some((
                            &["*.WAD", "*.wad", "*.deh", "*.pk3"],
                            "Supported files (*.WAD, *.wad, *.deh, *.pk3)",
                        )),
                    );
                    if let Some(new_path) = sel {
                        tab_config.input_paths[index] = new_path.clone();
                        tab_config.last_input_dir = Path::new(&new_path)
                            .parent()
                            .map(|d| d.to_string_lossy().to_string());
                        *store_config = true;
                    }
                }

                if ui
                    .add_sized(
                        egui::vec2(CONFIGURE_BUTTON_WIDTH + 1.0, ui.spacing().interact_size.y),
                        egui::Button::new("×"),
                    )
                    .clicked()
                {
                    input_path_indexes_to_remove.push(index);
                }

                ui.add(
                    egui::Label::new(
                        egui::RichText::new(path.clone())
                            .monospace()
                            .color(color_for_path(&path)),
                    )
                    .truncate(),
                );
            });
        }
    });
}

fn hsv_to_rgb_u8(h_deg: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = (h_deg % 360.0 + 360.0) % 360.0;
    let c = v * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

fn color_for_path(path: &str) -> egui::Color32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let seed = hasher.finish();

    let mut x = seed.wrapping_add(0x9E3779B97F4A7C15);
    x = x
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let hue = (x % 360) as f32;

    let (r, g, b) = hsv_to_rgb_u8(hue, PATH_COLOR_SAT, PATH_COLOR_VAL);
    egui::Color32::from_rgb(r, g, b)
}
