use crate::application::preferences::ColorScheme;
use crate::fl;
use crate::ui::preferences::messages::{PreferencesComponentInput, PreferencesComponentOutput};
use crate::ui::preferences::model::{PreferencesComponentInit, PreferencesComponentModel};
use mxl_player_components::ui::codec_ranking::{
    messages::CodecRankingComponentOutput,
    model::{CodecRankingComponentInit, CodecRankingComponentModel},
};
use mxl_relm4_components::relm4::{
    self, ComponentController, ComponentSender,
    adw::{self, prelude::*},
    component::{Component, ComponentParts},
    gtk,
};

const COLOR_SCHEME_DEFAULT_MODE: u32 = 0;
const COLOR_SCHEME_LIGHT_MODE: u32 = 1;
const COLOR_SCHEME_DARK_MODE: u32 = 2;

#[relm4::component(pub)]
impl Component for PreferencesComponentModel {
    type Init = PreferencesComponentInit;
    type Input = PreferencesComponentInput;
    type Output = PreferencesComponentOutput;
    type CommandOutput = ();

    view! {
        adw::PreferencesWindow {
            set_title: Some(&fl!("preferences")),
            set_hide_on_close: true,
            set_destroy_with_parent: true,
            set_height_request: 800,

            add = &adw::PreferencesPage {
                set_vexpand: true,

                add = &adw::PreferencesGroup {
                    set_title: &fl!("preferences", "general"),
                    adw::ActionRow {
                        set_title: &fl!("auto-play"),
                        set_subtitle: &fl!("auto-play", "description"),
                        set_activatable: true,
                        set_activatable_widget: Some(&auto_play_widget),
                        #[name(auto_play_widget)]
                        add_suffix = &gtk::Switch {
                            set_state: model.auto_play,
                            set_active: model.auto_play,
                            set_valign: gtk::Align::Center,
                            connect_state_notify[sender] => move |switch| {
                                sender.input(PreferencesComponentInput::SetAutoPlay(switch.state()));
                            }
                        },
                    },
                },

                add = &adw::PreferencesGroup {
                    set_title: &fl!("preferences", "appearance"),
                    adw::ComboRow {
                        set_title: &fl!("color-scheme"),
                        set_subtitle: &fl!("color-scheme", "description"),
                        set_model: Some(&gtk::StringList::new(&[
                            &fl!("color-scheme", "default"),
                            &fl!("color-scheme", "light"),
                            &fl!("color-scheme", "dark"),
                        ])),
                        set_selected: match model.color_scheme {
                            ColorScheme::Default => COLOR_SCHEME_DEFAULT_MODE,
                            ColorScheme::Light => COLOR_SCHEME_LIGHT_MODE,
                            ColorScheme::Dark => COLOR_SCHEME_DARK_MODE,
                        },
                        connect_selected_notify[sender] => move |combo_row| {
                            match combo_row.selected() {
                                COLOR_SCHEME_LIGHT_MODE => sender.input_sender().send(PreferencesComponentInput::SetColorScheme(ColorScheme::Light)).unwrap(),
                                COLOR_SCHEME_DARK_MODE => sender.input_sender().send(PreferencesComponentInput::SetColorScheme(ColorScheme::Dark)).unwrap(),
                                _ /*DEFAULT_MODE*/ => sender.input_sender().send(PreferencesComponentInput::SetColorScheme(ColorScheme::Default)).unwrap(),
                            }
                        },
                    },
                },

                add = model.decoder_ranking.widget(),
            }
        }
    }

    // Initialize the component.
    fn init(init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let model = PreferencesComponentModel {
            color_scheme: init.color_scheme,
            auto_play: init.auto_play,
            decoder_ranking: CodecRankingComponentModel::builder()
                .launch(CodecRankingComponentInit {
                    title: fl!("preferences", "video-decoder"),
                    codec_info_list: init.decoder_info,
                })
                .forward(sender.output_sender(), |out| match out {
                    CodecRankingComponentOutput::SetRank(name, rank) => {
                        PreferencesComponentOutput::DecoderRank(name, rank)
                    }
                }),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _: &Self::Root,
    ) {
        match msg {
            PreferencesComponentInput::SetColorScheme(scheme) => {
                sender
                    .output(PreferencesComponentOutput::ColorScheme(scheme))
                    .unwrap_or_default();
            }
            PreferencesComponentInput::SetAutoPlay(value) => {
                sender
                    .output(PreferencesComponentOutput::AutoPlay(value))
                    .unwrap_or_default();
            }
        }
        self.update_view(widgets, sender)
    }
}
