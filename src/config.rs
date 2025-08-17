use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabConfig {
    pub engine_path: Option<String>,
    pub iwad_path: Option<String>,
    pub input_paths: Vec<String>,
}

impl Default for TabConfig {
    fn default() -> Self {
        Self {
            engine_path: None,
            iwad_path: None,
            input_paths: Vec::new(),
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
    pub selected_tab: usize,
    pub last_engine_dir: Option<String>,
    pub last_iwad_dir: Option<String>,
    pub last_input_dir: Option<String>,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    #[serde(default)]
    pub title_mode: TitleMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tabs: vec![TabConfig::default()],
            selected_tab: 0,
            last_engine_dir: None,
            last_iwad_dir: None,
            last_input_dir: None,
            window_width: Some(640.0),
            window_height: Some(480.0),
            title_mode: TitleMode::default(),
        }
    }
}
