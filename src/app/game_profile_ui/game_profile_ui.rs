use crate::app::game_profile_ui::command_line_ui::command_line_ui;
use crate::app::game_profile_ui::game_engine_config_ui::game_engine_config_ui;
use crate::app::game_profile_ui::input_files_config_ui::input_files_config_ui;
use crate::app::game_profile_ui::iwad_config_ui::iwad_config_ui;
#[cfg(target_os = "linux")]
use crate::app::game_profile_ui::wrappers_ui::wrappers_ui;
use crate::config::{Config, MangohudMode, PrimeRunMode};
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
    let engine = tab_config.engine_path.as_ref()?;
    let iwad = tab_config.iwad_path.as_ref()?;

    let mut argv: Vec<&str> = Vec::new();

    if tab_config.use_mangohud && tab_config.mangohud_mode == MangohudMode::Bin {
        argv.push("mangohud");
    }

    if tab_config.use_prime_run && tab_config.prime_run_mode == PrimeRunMode::Bin {
        argv.push("prime-run");
    }

    if tab_config.use_umu_run {
        argv.push("umu-run");
    }

    argv.push(engine.as_str());
    argv.push("-iwad");
    argv.push(iwad.as_str());
    argv.push("-file");

    argv.extend(tab_config.input_paths.iter().map(String::as_str));

    let mut cmd = Command::new(argv[0]);
    cmd.args(&argv[1..]);

    if tab_config.use_umu_run {
        cmd.env("PROTONPATH", &tab_config.proton_runner);
    }

    if tab_config.use_mangohud && tab_config.mangohud_mode == MangohudMode::Env {
        cmd.env("MANGOHUD", "1");
    }

    if tab_config.use_prime_run && tab_config.prime_run_mode == PrimeRunMode::Env {
        cmd.env("__NV_PRIME_RENDER_OFFLOAD", "1")
            .env("__GLX_VENDOR_LIBRARY_NAME", "nvidia")
            .env("__VK_LAYER_NV_optimus", "NVIDIA_only");
    }

    Some(cmd)
}
