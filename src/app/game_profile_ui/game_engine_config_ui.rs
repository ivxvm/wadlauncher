use crate::config::Config;
use eframe::egui;
use std::path::Path;
use tinyfiledialogs as tfd;

const MIN_LABEL_WIDTH: f32 = 50.0;
const CONFIGURE_BUTTON_WIDTH: f32 = 16.0;

pub(super) fn game_engine_config_ui(ui: &mut egui::Ui, cfg: &mut Config, store_config: &mut bool) {
    ui.horizontal(|ui| {
        let tab_config = cfg.get_active_tab();
        ui.label("Game engine:");
        allocate_truncated_label_ui(ui, CONFIGURE_BUTTON_WIDTH, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(tab_config.engine_path.as_deref().unwrap_or("<Empty>"))
                        .monospace(),
                )
                .truncate(),
            )
        });
        if ui
            .add_sized(
                egui::vec2(CONFIGURE_BUTTON_WIDTH, ui.spacing().interact_size.y),
                egui::Button::new("..."),
            )
            .clicked()
        {
            let start_dir = tab_config
                .engine_path
                .as_ref()
                .and_then(|p| Path::new(p).parent().map(|d| d.to_str().unwrap_or(".")))
                .or(cfg.last_engine_dir.as_deref())
                .unwrap_or(".");
            let path = tfd::open_file_dialog("Select Game Engine", start_dir, None);
            if let Some(path) = path {
                cfg.get_active_tab_mut().engine_path = Some(path.clone());
                cfg.last_engine_dir = Path::new(&path)
                    .parent()
                    .map(|d| d.to_string_lossy().to_string());
                *store_config = true;
            }
        }
    });
}

fn allocate_truncated_label_ui<F, R>(
    ui: &mut egui::Ui,
    right_offset: f32,
    add_contents: F,
) -> egui::InnerResponse<R>
where
    F: FnOnce(&mut egui::Ui) -> R,
{
    let max_label_width = ui.available_width() - right_offset;
    ui.allocate_ui_with_layout(
        egui::vec2(
            max_label_width.max(MIN_LABEL_WIDTH),
            ui.spacing().interact_size.y,
        ),
        egui::Layout::left_to_right(egui::Align::Center),
        add_contents,
    )
}
