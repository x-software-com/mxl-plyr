use crate::application::preferences::ColorScheme;
use mxl_player_components::ui::codec_ranking::model::{CodecInfoList, CodecRankingComponentModel};
use mxl_relm4_components::relm4::Controller;

pub struct PreferencesComponentInit {
    pub color_scheme: ColorScheme,
    pub auto_play: bool,
    pub decoder_info: CodecInfoList,
}

pub struct PreferencesComponentModel {
    pub(super) color_scheme: ColorScheme,
    pub(super) auto_play: bool,
    pub(super) decoder_ranking: Controller<CodecRankingComponentModel>,
}
