use std::hash::Hash;

use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    pub id: Uuid,
    pub engine_path: Option<String>,
    pub iwad_path: Option<String>,
    pub input_paths: Vec<String>,
    pub last_input_dir: Option<String>,
    #[serde(default)]
    pub use_mangohud: bool,
    #[serde(default)]
    pub use_umu_run: bool,
    #[serde(default)]
    pub proton_runner: String,
}

impl Hash for TabConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            engine_path: None,
            iwad_path: None,
            input_paths: Vec::new(),
            last_input_dir: None,
            use_mangohud: false,
            use_umu_run: false,
            proton_runner: "".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TitleMode {
    Adaptive,
    Short,
    Long,
}

impl Default for TitleMode {
    fn default() -> Self {
        TitleMode::Adaptive
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tabs: Vec<TabConfig>,
    pub active_tab: Option<Uuid>,
    pub last_engine_dir: Option<String>,
    pub last_iwad_dir: Option<String>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    #[serde(default)]
    pub title_mode: TitleMode,
    #[serde(default)]
    pub show_command_line: bool,
    #[serde(default)]
    pub show_iwad_in_long_titles: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tabs: vec![TabConfig::default()],
            active_tab: None,
            last_engine_dir: None,
            last_iwad_dir: None,
            window_width: Some(640.0),
            window_height: Some(480.0),
            title_mode: TitleMode::default(),
            show_command_line: false,
            show_iwad_in_long_titles: false,
        }
    }
}

impl Config {
    pub fn get_active_tab_index(&self) -> Option<usize> {
        self.tabs.iter().position(|t| self.active_tab == Some(t.id))
    }

    pub fn get_active_tab(&self) -> &TabConfig {
        self.get_active_tab_index()
            .and_then(|i| self.tabs.get(i))
            .unwrap()
    }

    pub fn get_active_tab_mut(&mut self) -> &mut TabConfig {
        self.get_active_tab_index()
            .and_then(|i| self.tabs.get_mut(i))
            .unwrap()
    }
}
