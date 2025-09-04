use mxl_player_components::player::MaxLateness;
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DropFrames {
    drop_frames: bool,
}

#[allow(dead_code)]
impl DropFrames {
    pub fn value(&self) -> bool {
        self.drop_frames
    }

    pub fn set_value(&mut self, value: bool) {
        self.drop_frames = value;
    }

    pub fn qos() -> bool {
        false
    }

    pub fn max_lateness(&self) -> MaxLateness {
        if self.drop_frames {
            MaxLateness::Default
        } else {
            MaxLateness::Unlimited
        }
    }
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
    pub drop_frames: DropFrames,
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
            drop_frames: Default::default(),
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

use super::version2 as last_version;

impl From<last_version::WindowSize> for WindowSize {
    fn from(o: last_version::WindowSize) -> Self {
        Self {
            width: o.width,
            height: o.height,
        }
    }
}

impl From<last_version::ColorScheme> for ColorScheme {
    fn from(o: last_version::ColorScheme) -> Self {
        match o {
            last_version::ColorScheme::Default => Self::Default,
            last_version::ColorScheme::Dark => Self::Dark,
            last_version::ColorScheme::Light => Self::Light,
        }
    }
}

impl From<last_version::PreferencesData> for PreferencesData {
    fn from(o: last_version::PreferencesData) -> Self {
        Self {
            window_size: o.window_size.into(),
            show_flap: o.show_flap,
            color_scheme: o.color_scheme.into(),
            auto_play: o.auto_play,
            decoder_ignore_list: o.decoder_ignore_list,
            ..Default::default()
        }
    }
}
