use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: 1600,
            height: 900,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ColorScheme {
    #[default]
    Default,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesData {
    #[serde(default)]
    pub window_size: WindowSize,
    #[serde(default = "default_show_flap")]
    pub show_flap: bool,
    #[serde(default)]
    pub color_scheme: ColorScheme,
    #[serde(default = "default_auto_play")]
    pub auto_play: bool,
    #[serde(default)]
    pub decoder_ignore_list: Vec<String>,
}

impl Default for PreferencesData {
    fn default() -> Self {
        Self {
            window_size: Default::default(),
            show_flap: default_show_flap(),
            color_scheme: Default::default(),
            auto_play: default_auto_play(),
            decoder_ignore_list: Default::default(),
        }
    }
}

fn default_show_flap() -> bool {
    true
}

fn default_auto_play() -> bool {
    true
}
