use crate::config::{Config, TabConfig};
use eframe::egui;
use egui_dnd::dnd;
use std::path::Path;

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

fn build_long_titles(cfg: &Config) -> Vec<String> {
    cfg.tabs
        .iter()
        .map(|tab| {
            if tab.engine_path.is_none() && tab.iwad_path.is_none() && tab.input_paths.is_empty() {
                "New Tab".to_owned()
            } else {
                // Prefer showing the first input (wad) as the primary title, with engine/iwad in parentheses.
                let wad_name = tab.input_paths.get(0).map(|wad| {
                    sanitize_tab_name_part(
                        &Path::new(wad)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(wad),
                    )
                });
                let engine_name = tab.engine_path.as_ref().map(|engine| {
                    sanitize_tab_name_part(
                        &Path::new(engine)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(engine),
                    )
                });
                let iwad_name = tab.iwad_path.as_ref().map(|iwad| {
                    sanitize_tab_name_part(
                        &Path::new(iwad)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(iwad),
                    )
                });

                if let Some(wad) = wad_name {
                    let mut extras: Vec<String> = Vec::new();
                    if let Some(engine) = engine_name {
                        extras.push(engine);
                    }
                    if cfg.show_iwad_in_long_titles {
                        if let Some(iwad) = iwad_name {
                            extras.push(iwad);
                        }
                    }
                    if extras.is_empty() {
                        wad
                    } else {
                        format!("{} [{}]", wad, extras.join(", "))
                    }
                } else {
                    // Fallback: if there's no wad, prefer iwad as primary and show engine in parentheses.
                    if let Some(iwad) = iwad_name {
                        if let Some(engine) = engine_name {
                            format!("{} [{}]", iwad, engine)
                        } else {
                            iwad
                        }
                    } else if let Some(engine) = engine_name {
                        engine
                    } else {
                        "New Tab".to_owned()
                    }
                }
            }
        })
        .collect()
}

fn build_short_titles(cfg: &Config) -> Vec<String> {
    cfg.tabs
        .iter()
        .map(|tab| {
            tab.input_paths
                .get(0)
                .map(|wad| {
                    sanitize_tab_name_part(
                        &Path::new(wad)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(wad),
                    )
                })
                .unwrap_or_else(|| "New Tab".to_owned())
        })
        .collect()
}

fn compute_long_width(ui: &egui::Ui, long_titles: &[String]) -> f32 {
    let font_id = egui::TextStyle::Button.resolve(ui.style());
    let mut long_width = 0.0;
    for title in long_titles {
        let galley = ui.fonts_mut(|fonts| {
            fonts.layout_no_wrap(title.clone(), font_id.clone(), egui::Color32::WHITE)
        });
        long_width += galley.size().x + 16.0; // padding for button
    }
    long_width += (long_titles.len() as f32) * 32.0; // close buttons and spacing
    long_width += 32.0; // new tab button
    long_width += 80.0; // settings button width reserve
    long_width
}

pub fn tab_bar_ui(cfg: &mut Config, ui: &mut egui::Ui, store_config: &mut bool) {
    egui::Panel::top("tab_bar").show_inside(ui, |ui| {
        let long_titles = build_long_titles(cfg);
        let short_titles = build_short_titles(cfg);
        let long_titles_width = compute_long_width(ui, &long_titles);
        let use_short_titles = match cfg.title_mode {
            crate::config::TitleMode::Adaptive => long_titles_width > ui.available_width(),
            crate::config::TitleMode::Short => true,
            crate::config::TitleMode::Long => false,
        };

        // We'll reserve a small area on the right for the persistent Settings tab.
        let settings_area_width = 100.0; // pixels reserved on the right for Settings
        let left_area_width = (ui.available_width() - settings_area_width).max(0.0);

        ui.horizontal(|ui| {
            // Left area: normal tabs + new tab button
            ui.allocate_ui_with_layout(
                egui::vec2(left_area_width, ui.spacing().interact_size.y),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let tabs_count = cfg.tabs.len();
                    let mut closed_tab_index: Option<usize> = None;

                    dnd(ui, "tabs_dnd").show_vec(&mut cfg.tabs, |ui, item, handle, state| {
                        let tab_title = if use_short_titles {
                            &short_titles[state.index]
                        } else {
                            &long_titles[state.index]
                        };
                        let is_selected = cfg.selected_tab == Some(item.id);
                        handle.show_drag_cursor_on_hover(false).ui(ui, |ui| {
                            if ui.selectable_label(is_selected, tab_title).clicked() {
                                cfg.selected_tab = Some(item.id);
                                *store_config = true;
                            }
                            if tabs_count > 1 {
                                if ui.button("×").on_hover_text("Close tab").clicked() {
                                    closed_tab_index = Some(state.index);
                                }
                            }
                        });
                    });

                    if let Some(index) = closed_tab_index {
                        if index < tabs_count - 1 {
                            cfg.selected_tab = cfg.tabs.get(index).map(|t| t.id);
                        } else {
                            cfg.selected_tab = cfg.tabs.last().map(|t| t.id);
                        }

                        *store_config = true;
                    }

                    if ui.button("+").on_hover_text("New tab").clicked() {
                        cfg.tabs.push(TabConfig::default());
                        cfg.selected_tab = cfg.tabs.last().map(|t| t.id);
                        *store_config = true;
                    }
                },
            );

            ui.add_space(ui.available_width() - settings_area_width);

            // Right area: Settings button (persistent, no close button)
            ui.allocate_ui_with_layout(
                egui::vec2(settings_area_width, ui.spacing().interact_size.y),
                egui::Layout::right_to_left(egui::Align::Max),
                |ui| {
                    let settings_selected = cfg.selected_tab == None;
                    if ui.selectable_label(settings_selected, "SETTINGS").clicked() {
                        cfg.selected_tab = None;
                        *store_config = true;
                    }
                },
            );
        });
    });
}
