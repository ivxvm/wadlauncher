use crate::config::{Config, TitleMode};
use eframe::egui;

/// Renders the Settings UI. Provides Title Mode dropdown and Show Command Line checkbox.
pub fn settings_ui(ctx: &egui::Context, cfg: &mut Config, store_config: &mut bool) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Tab title mode:");

            let mut current = cfg.title_mode;

            egui::ComboBox::from_label("")
                .selected_text(match current {
                    TitleMode::Adaptive => "Adaptive",
                    TitleMode::Short => "Short",
                    TitleMode::Long => "Long",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut current, TitleMode::Adaptive, "Adaptive");
                    ui.selectable_value(&mut current, TitleMode::Short, "Short");
                    ui.selectable_value(&mut current, TitleMode::Long, "Long");
                });

            if current != cfg.title_mode {
                cfg.title_mode = current;
                *store_config = true;
            }
        });

        ui.horizontal(|ui| {
            let mut show_iwad = cfg.show_iwad_in_long_titles;
            if ui
                .checkbox(&mut show_iwad, "Show IWAD in long titles")
                .changed()
            {
                cfg.show_iwad_in_long_titles = show_iwad;
                *store_config = true;
            }
        });

        ui.horizontal(|ui| {
            let mut show = cfg.show_command_line;
            if ui.checkbox(&mut show, "Show command line").changed() {
                cfg.show_command_line = show;
                *store_config = true;
            }
        });
    });
}
