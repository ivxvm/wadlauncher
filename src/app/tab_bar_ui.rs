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
    long_width
}

pub fn tab_bar_ui(cfg: &mut Config, ctx: &egui::Context, store_config: &mut bool) {
    egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
        let long_titles = build_long_titles(cfg);
        let short_titles = build_short_titles(cfg);
        let long_titles_width = compute_long_width(ui, &long_titles);
        let use_short_titles = long_titles_width > ui.available_width();
        ui.horizontal(|ui| {
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
                        cfg.tabs.remove(i);
                        if cfg.selected_tab >= cfg.tabs.len() {
                            cfg.selected_tab = cfg.tabs.len() - 1;
                        }
                        *store_config = true;
                        break;
                    }
                }
            }
            if ui.button("+").on_hover_text("New tab").clicked() {
                cfg.tabs.push(TabConfig::default());
                cfg.selected_tab = cfg.tabs.len() - 1;
                *store_config = true;
            }
        });
    });
}
