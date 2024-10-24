use crate::application::preferences::ColorScheme;

#[derive(Debug)]
pub enum PreferencesComponentInput {
    SetColorScheme(ColorScheme),
    SetAutoPlay(bool),
}

#[derive(Debug)]
pub enum PreferencesComponentOutput {
    ColorScheme(ColorScheme),
    AutoPlay(bool),
    DecoderRank(String, gst::Rank),
}
