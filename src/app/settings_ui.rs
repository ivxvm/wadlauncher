use crate::config::{Config, TitleMode};
use eframe::egui;

/// Renders the Settings UI. Provides Title Mode dropdown and Show Command Line checkbox.
pub fn settings_ui(ctx: &egui::Context, cfg: &mut Config, store_config: &mut bool) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Settings");
        ui.separator();

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

        ui.label("Adaptive will pick short vs long based on available width.");

        ui.separator();

        ui.horizontal(|ui| {
            let mut show = cfg.show_command_line;
            if ui.checkbox(&mut show, "Show command line").changed() {
                cfg.show_command_line = show;
                *store_config = true;
            }
        });
    });
}
