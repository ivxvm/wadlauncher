use crate::app::game_profile_ui::command_line_ui::command_line_ui;
use crate::app::game_profile_ui::game_engine_config_ui::game_engine_config_ui;
use crate::app::game_profile_ui::input_files_config_ui::input_files_config_ui;
use crate::app::game_profile_ui::iwad_config_ui::iwad_config_ui;
#[cfg(target_os = "linux")]
use crate::app::game_profile_ui::wrappers_ui::wrappers_ui;
use crate::config::Config;
use arboard::Clipboard;
use eframe::egui;
use std::process::Command;

pub fn game_profile_ui(
    titlepic_texture: &Option<egui::TextureHandle>,
    clipboard: &mut Clipboard,
    ui: &mut egui::Ui,
    cfg: &mut Config,
    input_path_indexes_to_remove: &mut Vec<usize>,
    iwad_to_load: &mut Option<String>,
    store_config: &mut bool,
) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        render_background(ui, titlepic_texture);
        game_engine_config_ui(ui, cfg, store_config);
        iwad_config_ui(ui, cfg, iwad_to_load, store_config);
        input_files_config_ui(ui, cfg, input_path_indexes_to_remove, store_config);
        #[cfg(target_os = "linux")]
        wrappers_ui(ui, cfg, store_config);
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

fn build_cmd(cfg: &Config) -> Option<Command> {
    let tab_config = cfg.get_active_tab();
    if let (Some(engine), Some(iwad)) = (
        tab_config.engine_path.as_ref(),
        tab_config.iwad_path.as_ref(),
    ) {
        let mut cmd = if tab_config.use_umu_run {
            let mut cmd = Command::new("umu-run");
            cmd.env("PROTONPATH", &tab_config.proton_runner);
            cmd.arg(engine);
            cmd
        } else {
            Command::new(engine)
        };

        if tab_config.use_mangohud {
            cmd.env("MANGOHUD", "1");
        }

        cmd.arg("-iwad")
            .arg(iwad)
            .arg("-file")
            .args(&tab_config.input_paths);

        Some(cmd)
    } else {
        None
    }
}
