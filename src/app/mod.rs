mod central_panel_ui;
mod tab_bar_ui;

use arboard::Clipboard;
use eframe::egui;
use eframe::egui::ColorImage;

use crate::config::Config;
use crate::wad::{
    decode_htitle, decode_titlepic, get_titlepic_dimensions, load_playpal_lump, load_titlepic_lump,
};

pub struct App {
    pub config: Config,
    pub clipboard: Clipboard,
    pub titlepic_texture: Option<egui::TextureHandle>,
    pub last_iwad_path: Option<String>,
    pub last_wad_path: Option<String>,
}

impl App {
    pub fn load_titlepic(
        &mut self,
        ctx: &egui::Context,
        iwad_path: Option<&str>,
        wad_path: Option<&str>,
    ) -> Option<()> {
        self.titlepic_texture = None;
        let palette = load_playpal_lump(iwad_path, wad_path)?;
        let (lump_name, titlepic) = load_titlepic_lump(iwad_path, wad_path)?;
        let (width, height, img) = match lump_name.as_str() {
            "TITLE" | "HTITLE" => {
                let img = decode_htitle(&titlepic, &palette)
                    .or_else(|| decode_titlepic(&titlepic, &palette, 320, 200))?;
                (320, 200, img)
            }
            "TITLEPIC" | _ => {
                let (width, height) = get_titlepic_dimensions(&titlepic);
                let img = decode_titlepic(&titlepic, &palette, width, height)?;
                (width, height, img)
            }
        };
        let color_img = ColorImage::from_rgba_unmultiplied([width, height], &img);
        self.titlepic_texture =
            Some(ctx.load_texture("titlepic", color_img, egui::TextureOptions::default()));
        Some(())
    }

    /// Handles window resize and persists new size to config. Returns true if config changed.
    fn handle_window_resize(&mut self, ctx: &egui::Context) -> bool {
        let win_size = ctx.available_rect().size();
        if self.config.window_width != Some(win_size.x)
            || self.config.window_height != Some(win_size.y)
        {
            self.config.window_width = Some(win_size.x);
            self.config.window_height = Some(win_size.y);
            return true;
        }
        false
    }

    /// Checks if TITLEPIC needs to be reloaded and reloads if needed.
    fn reload_titlepic_if_needed(&mut self, ctx: &egui::Context) {
        let cfg = &self.config;
        let tab_config = &cfg.tabs[cfg.selected_tab];
        let iwad_path = tab_config.iwad_path.clone();
        let wad_path = tab_config.input_paths.get(0).cloned();
        let mut need_titlepic = false;
        if wad_path.is_some() || iwad_path.is_some() {
            need_titlepic = self.last_iwad_path.as_deref() != iwad_path.as_deref()
                || self.last_wad_path.as_deref() != wad_path.as_deref()
                || self.titlepic_texture.is_none();
        }
        if need_titlepic {
            self.load_titlepic(ctx, iwad_path.as_deref(), wad_path.as_deref());
            self.last_iwad_path = iwad_path;
            self.last_wad_path = wad_path;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let mut should_store_config = self.handle_window_resize(ctx);
        self.reload_titlepic_if_needed(ctx);
        let mut input_path_indexes_to_remove = Vec::new();
        let mut iwad_to_load: Option<String> = None;
        tab_bar_ui::tab_bar_ui(&mut self.config, ctx, &mut should_store_config);
        let cfg = &mut self.config;
        central_panel_ui::central_panel_ui(
            &self.titlepic_texture,
            &mut self.clipboard,
            ctx,
            cfg,
            &mut input_path_indexes_to_remove,
            &mut iwad_to_load,
            &mut should_store_config,
        );
        let tab_config = cfg.tabs.get_mut(cfg.selected_tab).unwrap();
        input_path_indexes_to_remove.sort();

        for index in input_path_indexes_to_remove.iter().rev() {
            tab_config.input_paths.remove(*index);
            should_store_config = true;
        }

        if should_store_config {
            confy::store("wadlauncher", None, &*cfg).unwrap();
        }
    }
}
