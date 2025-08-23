use crate::config::Config;
use arboard::Clipboard;
use eframe::egui;
use std::path::Path;
use std::process::Command;
use tinyfiledialogs as tfd;

const MIN_LABEL_WIDTH: f32 = 50.0;
const CONFIGURE_BUTTON_WIDTH: f32 = 16.0;

// Input path label color parameters (HSV). Hue is generated per-path from a seeded PRNG,
// saturation and value are fixed constants.
const PATH_COLOR_SAT: f32 = 0.5; // 0.0..1.0
const PATH_COLOR_VAL: f32 = 0.6; // 0.0..1.0

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

    // Stable-ish hash of the path string
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let seed = hasher.finish();

    // Small LCG to produce a pseudo-random hue from the seed
    let mut x = seed.wrapping_add(0x9E3779B97F4A7C15);
    x = x
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let hue = (x % 360) as f32;

    let (r, g, b) = hsv_to_rgb_u8(hue, PATH_COLOR_SAT, PATH_COLOR_VAL);
    egui::Color32::from_rgb(r, g, b)
}

fn allocate_truncated_label_ui<F, R>(
    ui: &mut egui::Ui,
    right_offset: f32,
    add_contents: F,
) -> egui::InnerResponse<R>
where
    F: FnOnce(&mut egui::Ui) -> R,
{
    let max_label_width = ui.available_width() - right_offset;
    ui.allocate_ui_with_layout(
        egui::vec2(
            max_label_width.max(MIN_LABEL_WIDTH),
            ui.spacing().interact_size.y,
        ),
        egui::Layout::left_to_right(egui::Align::Center),
        add_contents,
    )
}

fn render_background(ui: &mut egui::Ui, titlepic_texture: &Option<egui::TextureHandle>) {
    if let Some(tex) = titlepic_texture {
        ui.painter().image(
            tex.id(),
            ui.max_rect(),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}

fn game_engine_config_ui(ui: &mut egui::Ui, cfg: &mut Config, store_config: &mut bool) {
    let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();
    ui.horizontal(|ui| {
        ui.label("Game engine:");
        allocate_truncated_label_ui(ui, CONFIGURE_BUTTON_WIDTH, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(tab_config.engine_path.as_deref().unwrap_or("<Empty>"))
                        .monospace(),
                )
                .truncate(),
            )
        });
        if ui
            .add_sized(
                egui::vec2(CONFIGURE_BUTTON_WIDTH, ui.spacing().interact_size.y),
                egui::Button::new("..."),
            )
            .clicked()
        {
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
                *store_config = true;
            }
        }
    });
}

fn iwad_config_ui(
    ui: &mut egui::Ui,
    cfg: &mut Config,
    iwad_to_load: &mut Option<String>,
    store_config: &mut bool,
) {
    let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();
    ui.horizontal(|ui| {
        ui.label("IWAD:");
        allocate_truncated_label_ui(ui, CONFIGURE_BUTTON_WIDTH, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(tab_config.iwad_path.as_deref().unwrap_or("<Empty>"))
                        .monospace(),
                )
                .truncate(),
            )
        });
        if ui
            .add_sized(
                egui::vec2(CONFIGURE_BUTTON_WIDTH, ui.spacing().interact_size.y),
                egui::Button::new("..."),
            )
            .clicked()
        {
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
                *store_config = true;
                *iwad_to_load = Some(path);
            }
        }
    });
}

fn input_files_config_ui(
    ui: &mut egui::Ui,
    cfg: &mut Config,
    input_path_indexes_to_remove: &mut Vec<usize>,
    store_config: &mut bool,
) {
    let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();
    ui.horizontal(|ui| {
        ui.label("Input files:");
        if ui.button("Add").clicked() {
            let start_dir = tab_config
                .last_input_dir
                .as_deref()
                .or_else(|| {
                    tab_config
                        .input_paths
                        .last()
                        .map(|last| Path::new(last).parent().map(|d| d.to_str().unwrap_or(".")))
                        .flatten()
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
                tab_config.input_paths.push(path.clone());
                tab_config.last_input_dir = Path::new(&path)
                    .parent()
                    .map(|d| d.to_string_lossy().to_string());
                *store_config = true;
            }
        }
    });
    ui.group(|ui| {
        if tab_config.input_paths.is_empty() {
            ui.label("<Empty>");
        }
        let initial_len = tab_config.input_paths.len();
        for index in 0..initial_len {
            // Clone the path for display so we don't hold an immutable borrow while mutating the vec.
            let path = tab_config.input_paths[index].clone();
            ui.horizontal(|ui| {
                // Move up button (disabled for first item)
                if ui
                    .add_enabled(index > 0, egui::Button::new("/\\"))
                    .clicked()
                {
                    tab_config.input_paths.swap(index, index - 1);
                    *store_config = true;
                }

                // Move down button (disabled for last item)
                let last_idx = tab_config.input_paths.len().saturating_sub(1);
                if ui
                    .add_enabled(index < last_idx, egui::Button::new("\\/"))
                    .clicked()
                {
                    tab_config.input_paths.swap(index, index + 1);
                    *store_config = true;
                }

                // Replace file button: open dialog in the same folder as this input file
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

fn build_cmd(cfg: &Config) -> Option<Command> {
    let tab_config = &cfg.tabs[cfg.selected_tab];
    if let (Some(engine), Some(iwad)) = (
        tab_config.engine_path.as_ref(),
        tab_config.iwad_path.as_ref(),
    ) {
        let mut cmd = Command::new(engine);
        cmd.arg("-iwad")
            .arg(iwad)
            .arg("-file")
            .args(&tab_config.input_paths);
        Some(cmd)
    } else {
        None
    }
}

fn command_line_ui(ui: &mut egui::Ui, clipboard: &mut Clipboard, cmd: &Option<Command>) {
    let cmd_str = cmd
        .as_ref()
        .map(|cmd| format!("{:?}", cmd))
        .unwrap_or("<Incomplete command>".to_string());
    ui.horizontal(|ui| {
        ui.label("Command line:");
        if ui.button("Copy").clicked() {
            clipboard.set_text(cmd_str.clone()).unwrap();
        }
    });
    ui.group(|ui| {
        ui.label(cmd_str.clone());
    });
}

pub fn game_profile_ui(
    titlepic_texture: &Option<egui::TextureHandle>,
    clipboard: &mut Clipboard,
    ctx: &egui::Context,
    cfg: &mut Config,
    input_path_indexes_to_remove: &mut Vec<usize>,
    iwad_to_load: &mut Option<String>,
    store_config: &mut bool,
) {
    egui::CentralPanel::default().show(ctx, |ui| {
        render_background(ui, titlepic_texture);
        game_engine_config_ui(ui, cfg, store_config);
        iwad_config_ui(ui, cfg, iwad_to_load, store_config);
        input_files_config_ui(ui, cfg, input_path_indexes_to_remove, store_config);
        let cmd = build_cmd(cfg);
        if cfg.show_command_line {
            command_line_ui(ui, clipboard, &cmd);
        }
        if ui.button("Launch").clicked() {
            if let Some(mut cmd) = cmd {
                println!("Launching game:\n{:?}\n", cmd);
                cmd.spawn().unwrap();
            }
        }
    });
}
