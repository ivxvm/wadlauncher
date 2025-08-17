use crate::config::{Config, TabConfig};
use eframe::egui;
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
                let mut title = String::new();
                if let Some(engine) = &tab.engine_path {
                    title.push_str(&sanitize_tab_name_part(
                        &Path::new(engine)
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
                        &Path::new(iwad)
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
                        &Path::new(wad)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or(wad),
                    ));
                }
                title
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
        let galley = ui.fonts(|fonts| {
            fonts.layout_no_wrap(title.clone(), font_id.clone(), egui::Color32::WHITE)
        });
        long_width += galley.size().x + 16.0; // padding for button
    }
    long_width += (long_titles.len() as f32) * 32.0; // close buttons and spacing
    long_width += 32.0; // new tab button
    long_width += 80.0; // settings button width reserve
    long_width
}

pub fn tab_bar_ui(cfg: &mut Config, ctx: &egui::Context, store_config: &mut bool) {
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
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
                    for (i, _tab) in cfg.tabs.iter().enumerate() {
                        let tab_title = if use_short_titles {
                            &short_titles[i]
                        } else {
                            &long_titles[i]
                        };
                        let selected = i == cfg.selected_tab;
                        if ui.selectable_label(selected, tab_title).clicked() {
                            cfg.selected_tab = i;
                            *store_config = true;
                        }
                        if cfg.tabs.len() > 1 {
                            if ui.button("×").on_hover_text("Close tab").clicked() {
                                // Preserve settings selection if it was selected (settings index == old_len)
                                let prev_selected = cfg.selected_tab;
                                let old_len = cfg.tabs.len();
                                // Remove the tab
                                cfg.tabs.remove(i);
                                let new_len = cfg.tabs.len();

                                let settings_index_before = old_len; // settings index was equal to number of tabs
                                let was_settings_selected = prev_selected == settings_index_before;

                                if was_settings_selected {
                                    // Keep Settings selected; its index is now new_len
                                    cfg.selected_tab = new_len;
                                } else {
                                    // Adjust selected index based on relationship to the removed tab
                                    if prev_selected == i {
                                        // The currently selected tab was closed.
                                        // Prefer the tab that shifted into this index; if the closed tab was the last,
                                        // select the new last tab.
                                        if i >= new_len {
                                            cfg.selected_tab = new_len.saturating_sub(1);
                                        } else {
                                            cfg.selected_tab = i;
                                        }
                                    } else if i < prev_selected {
                                        // A tab before the selected one was removed, shift selection left by one.
                                        cfg.selected_tab = prev_selected.saturating_sub(1);
                                    } else {
                                        // A tab after the selected one was removed, selection unchanged.
                                        cfg.selected_tab = prev_selected;
                                    }
                                }

                                *store_config = true;
                                break;
                            }
                        }
                    }
                    if ui.button("+").on_hover_text("New tab").clicked() {
                        cfg.tabs.push(TabConfig::default());
                        cfg.selected_tab = cfg.tabs.len().saturating_sub(1);
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
                    let settings_tab_index = cfg.tabs.len();
                    let settings_selected = cfg.selected_tab == settings_tab_index;
                    if ui.selectable_label(settings_selected, "SETTINGS").clicked() {
                        cfg.selected_tab = settings_tab_index;
                        *store_config = true;
                    }
                },
            );
        });
    });
}
