use crate::application::preferences::ColorScheme;

#[derive(Debug)]
pub enum PreferencesComponentInput {
    ColorScheme(ColorScheme),
    AutoPlay(bool),
    DropFrames(bool),
}

#[derive(Debug)]
pub enum PreferencesComponentOutput {
    ColorScheme(ColorScheme),
    AutoPlay(bool),
    DropFrames(bool),
    DecoderRank(String, gst::Rank),
}
