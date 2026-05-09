use crate::config::{Config, MangohudMode};
use eframe::egui;
use std::fs;
use std::sync::{Mutex, MutexGuard, OnceLock};

static PROTON_RUNNERS: OnceLock<Mutex<Vec<ProtonRunner>>> = OnceLock::new();

const WRAPPER_CHECKBOX_WIDTH: f32 = 88.0;

struct ProtonRunner {
    name: String,
    path: String,
}

pub(super) fn wrappers_ui(ui: &mut egui::Ui, cfg: &mut Config, store_config: &mut bool) {
    let tab_config = cfg.get_active_tab_mut();

    let proton_runners = get_installed_proton_runners_cache();

    if proton_runners.is_empty() && !tab_config.proton_runner.is_empty() {
        tab_config.proton_runner = String::new();
    }

    ui.label("Wrappers:");

    ui.group(|ui| {
        ui.horizontal(|ui| {
            let umu_run_response =
                wrapper_checkbox_ui(ui, |ui| ui.checkbox(&mut tab_config.use_umu_run, "umu-run"));

            if umu_run_response.changed() {
                *store_config = true;
            }

            if proton_runners.is_empty() {
                ui.add_enabled_ui(false, |ui| {
                    egui::ComboBox::from_label("")
                        .selected_text("<No Proton runners>")
                        .show_ui(ui, |_| {});
                });
            } else {
                let mut selected_index = proton_runners
                    .iter()
                    .position(|r| *r.path == tab_config.proton_runner)
                    .unwrap_or_default();

                let selected_before = selected_index;

                egui::ComboBox::from_label("")
                    .selected_text(proton_runners[selected_index].name.as_str())
                    .show_ui(ui, |ui| {
                        for (index, option) in proton_runners.iter().enumerate() {
                            ui.selectable_value(&mut selected_index, index, option.name.as_str());
                        }
                    });

                if selected_index != selected_before || tab_config.proton_runner.is_empty() {
                    tab_config.proton_runner = proton_runners
                        .get(selected_index)
                        .map(|runner| runner.path.clone())
                        .unwrap_or_default();
                    *store_config = true;
                }
            }

            if ui.button("Refresh").clicked() {
                refresh_installed_proton_runners_cache();

                let proton_runners = get_installed_proton_runners_cache();

                if proton_runners
                    .iter()
                    .find(|r| &*r.path == &tab_config.proton_runner)
                    .is_none()
                {
                    tab_config.proton_runner = String::new();
                }
            }
        });

        ui.horizontal(|ui| {
            let mangohud_response = wrapper_checkbox_ui(ui, |ui| {
                ui.checkbox(&mut tab_config.use_mangohud, "mangohud")
            });

            if mangohud_response.changed() {
                *store_config = true;
            }

            let mut current = tab_config.mangohud_mode;

            egui::ComboBox::from_label("")
                .selected_text(match current {
                    MangohudMode::Env => "Env",
                    MangohudMode::Bin => "Bin",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut current, MangohudMode::Env, "Env");
                    ui.selectable_value(&mut current, MangohudMode::Bin, "Bin");
                });

            if current != tab_config.mangohud_mode {
                tab_config.mangohud_mode = current;
                *store_config = true;
            }
        });
    });
}

fn wrapper_checkbox_ui(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> egui::Response,
) -> egui::Response {
    let response = add_contents(ui);
    let remaining_width = WRAPPER_CHECKBOX_WIDTH - response.rect.width();

    if remaining_width > 0.0 {
        ui.add_space(remaining_width);
    }

    response
}

fn get_installed_proton_runners_cache() -> MutexGuard<'static, Vec<ProtonRunner>> {
    PROTON_RUNNERS
        .get_or_init(|| std::sync::Mutex::new(find_installed_proton_runners()))
        .lock()
        .unwrap()
}

fn refresh_installed_proton_runners_cache() {
    let cache = PROTON_RUNNERS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut options) = cache.lock() {
        *options = find_installed_proton_runners();
    }
}

fn find_installed_proton_runners() -> Vec<ProtonRunner> {
    let home = std::env::var("HOME").unwrap_or_default();

    if home.is_empty() {
        return Vec::new();
    }

    let mut options = Vec::new();

    let compatibilitytools_dir = format!("{home}/.steam/steam/compatibilitytools.d");

    if let Ok(entries) = fs::read_dir(compatibilitytools_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    options.push(ProtonRunner {
                        name: name.to_string(),
                        path: entry.path().to_str().unwrap_or_default().to_owned(),
                    });
                }
            }
        }
    }

    let common_dir = format!("{home}/.steam/steam/steamapps/common");

    if let Ok(entries) = fs::read_dir(common_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                if name.contains("Proton") {
                    options.push(ProtonRunner {
                        name: name.to_string(),
                        path: entry.path().to_str().unwrap_or_default().to_owned(),
                    });
                }
            }
        }
    }

    options.into_iter().collect()
}
