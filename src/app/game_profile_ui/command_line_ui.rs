use arboard::Clipboard;
use eframe::egui;
use std::process::Command;

pub(super) fn command_line_ui(ui: &mut egui::Ui, clipboard: &mut Clipboard, cmd: &Option<Command>) {
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
