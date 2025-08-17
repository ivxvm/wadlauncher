use crate::config::Config;
use eframe::egui;

/// Renders the Settings UI. Currently a simple placeholder.
pub fn settings_ui(ctx: &egui::Context, _cfg: &mut Config, _store_config: &mut bool) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Settings");
            ui.label("Placeholder: application settings will be shown here.");
        });
    });
}
