use crate::config::Config;
use arboard::Clipboard;
use eframe::egui;
use std::path::Path;
use std::process::Command;
use tinyfiledialogs as tfd;

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
        let prefix_label_response = ui.label("IWAD:");
        let prefix_label_width = prefix_label_response.rect.width();
        let button_width = 65.0;
        let overflow = 30.0;
        let max_path_label_width =
            ui.available_width() - (prefix_label_width + button_width) + overflow;
        ui.allocate_ui_with_layout(
            egui::vec2(max_path_label_width.max(50.0), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(tab_config.iwad_path.as_deref().unwrap_or("<Empty>"))
                            .monospace(),
                    )
                    .truncate(),
                )
            },
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
                *store_config = true;
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
}

fn build_cmd(cfg: &Config) -> Option<Command> {
    let tab_config = &cfg.tabs[cfg.selected_tab];
    if let (Some(engine), Some(iwad)) = (
        tab_config.engine_path.as_ref(),
        tab_config.iwad_path.as_ref(),
    ) {
        let mut cmd = Command::new(engine);
        cmd.args(&tab_config.input_paths).arg("-iwad").arg(iwad);
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
