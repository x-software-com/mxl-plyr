use anyhow::Context;
use gst::prelude::PluginFeatureExtManual;
use log::*;
use mxl_investigator::{
    create_report_dialog::{
        messages::CreateReportDialogInput,
        model::{CreateReportDialog, CreateReportDialogInit},
    },
    problem_report_dialog::{
        messages::{ProblemReportDialogInput, ProblemReportDialogOutput},
        model::{ProblemReportDialog, ProblemReportDialogInit},
    },
};
use mxl_player_components::{
    actions::{self, Accelerators},
    gst::prelude::*,
    gst_play::PlayMediaInfo,
    ui::{
        message_dialog::{
            messages::{MessageDialogInput, MessageDialogOutput, MessageDialogType},
            model::MessageDialog,
        },
        player::{
            messages::{PlaybackState, PlayerComponentInput, PlayerComponentOutput, Track},
            model::{PlayerComponentInit, PlayerComponentModel},
        },
        playlist::{
            messages::{PlaylistComponentInput, PlaylistComponentOutput, PlaylistState},
            model::{PlaylistComponentInit, PlaylistComponentModel},
        },
        video_offsets_dialog::{
            messages::{VideoOffsetsComponentInput, VideoOffsetsComponentOutput},
            model::{VideoOffsetsComponentInit, VideoOffsetsComponentModel},
        },
    },
};
use mxl_relm4_components::{
    relm4::{
        self,
        actions::*,
        adw::{Toast, prelude::*},
        component::Connector,
        gtk::{gio, glib},
        prelude::*,
    },
    relm4_components::open_dialog::*,
    third_party_licenses_dialog::model::ThirdPartyLicensesComponentModel,
};
use relm4_icons::icon_names;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use glib::clone;

use crate::{
    about,
    application::{
        self,
        preferences::{ColorScheme, PreferencesManager, WindowSize},
    },
    fl,
    ui::preferences::{
        messages::PreferencesComponentOutput,
        model::{PreferencesComponentInit, PreferencesComponentModel},
    },
};

pub struct AppInit {
    pub compositor: Option<mxl_player_components::gst::Element>,
    pub uris: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Next,
    Previous,
    Stopped,
    Paused,
    Playing,
    Buffering,
    Error,
}

pub struct App {
    request_exit: bool,
    ready_to_exit: Arc<Mutex<bool>>,
    preferences: PreferencesManager,
    window_title: String,
    current_position: f64,
    duration: f64,
    volume: f64,
    speed: f64,
    app_state: AppState,
    auto_start_done: bool,
    reload_player_on_stopped: bool,
    audio_track_items: Vec<(usize, String)>,
    playlist_component: Controller<PlaylistComponentModel>,
    player_component: Controller<PlayerComponentModel>,
    message_dialog: Controller<MessageDialog>,
    problem_report_dialog: Controller<ProblemReportDialog>,
    create_report_dialog: Controller<CreateReportDialog>,
    file_open_dialog: Controller<OpenDialogMulti>,
    video_offsets_dialog: Controller<VideoOffsetsComponentModel>,
    about_dialog: adw::AboutWindow,
    preferences_dialog: Controller<PreferencesComponentModel>,
    third_party_licenses_dialog: Connector<ThirdPartyLicensesComponentModel>,
    update_actions: Vec<Box<dyn Fn(AppState)>>,
    audio_track_menu: gio::Menu,
}

#[derive(Debug)]
pub struct WarnMsg {
    title: String,
    warning: String,
    informative: Option<String>,
}

#[derive(Debug)]
pub enum AppMsg {
    PlayerInitialized,
    TogglePlaylistVisibility,
    ShowPlaylist(bool),
    ShowPlaylistChanged(bool),
    TogglePlayPause,
    ToggleFullScreen,
    DumpPipeline,
    Stop,
    Stopped,
    Seek(f64),
    NextFrame,
    SwitchAudioTrack(String),
    IncreaseVolume,
    DecreaseVolume,
    ResetVolume,
    SetVolume(f64),
    ChangeVolume(f64),
    IncreaseSpeed,
    DecreaseSpeed,
    ResetSpeed,
    SetSpeed(f64),
    ChangeSpeed(f64),
    SetAudioVideoOffset(i64),
    SetSubtitleVideoOffset(i64),
    SwitchUri(String),
    Previous,
    Next,
    PlayerMediaInfoUpdated(PlayMediaInfo),
    FileChooserRequest,
    FileChooserAccept(Vec<PathBuf>),
    FileChooserIgnore,
    ShowAboutDialog,
    ShowVideoOffsetsDialog,
    ShowPreferencesDialog,
    ShowThirdPartyLicensesDialog,
    Quit,
    SaveWindowState { width: i32, height: i32 },
    ShowPlaybackView,
    PlaybackError(anyhow::Error),
    Warning(WarnMsg),
    ProblemReportDialogOpen,
    ProblemReportDialogClosed,
    CreateReportDialogOpen,
    DoAutoStart,
}

#[derive(Debug)]
pub enum AppCmd {
    PlayerInitialized(Option<anyhow::Error>),
    PlayerMediaInfoUpdated(PlayMediaInfo),
    PlayerDurationChanged(f64),
    PlayerPositionUpdated(f64),
    PlayerSeekDone,
    PlayerEndOfStream(String),
    PlayerStateChanged(Option<PlaybackState>, PlaybackState),
    PlayerVolumeChanged(f64),
    PlayerSpeedChanged(f64),
    PlayerAudioVideoOffsetChanged(i64),
    PlayerSubtitleVideoOffsetChanged(i64),
    PlayerWarning(anyhow::Error),
    PlayerError(anyhow::Error),
    PlaylistChanged,
    PlaylistSwitchUri(String),
    PlaylistEndOfPlaylist,
    PlaylistStateChanged(PlaylistState),
    PlaylistFileChooserRequest,
    PreferencesSetColorScheme(ColorScheme),
    PreferencesSetAutoPlay(bool),
    PreferencesSetDecoderRank(String, gst::Rank),
    VideoOffsetsDialogAudioVideoOffsetChanged(i64),
    VideoOffsetsDialogSubtitleVideoOffsetChanged(i64),
    MessageDialogCreateReport,
    MessageDialogQuitApp,
}

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(Quit, AppActionGroup, "quit");

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(About, WindowActionGroup, "about");
relm4::new_stateless_action!(CreateReport, WindowActionGroup, "create-report");
relm4::new_stateless_action!(ThirdPartyLicenses, WindowActionGroup, "third-party-licenses");
relm4::new_stateless_action!(
    TogglePlaylistVisibility,
    WindowActionGroup,
    "toggle-playlist-visibility"
);
relm4::new_stateless_action!(Preferences, WindowActionGroup, "preferences");
relm4::new_stateless_action!(VideoOffsets, WindowActionGroup, "video-offsets");
relm4::new_stateless_action!(FileChooser, WindowActionGroup, "file-chooser");
relm4::new_stateless_action!(TogglePlayPause, WindowActionGroup, "toggle-play-pause");
relm4::new_stateless_action!(NextFrame, WindowActionGroup, "next-frame");
relm4::new_stateless_action!(Stop, WindowActionGroup, "stop");
relm4::new_stateless_action!(NextUri, WindowActionGroup, "next-uri");
relm4::new_stateless_action!(PrevUri, WindowActionGroup, "prev-uri");
relm4::new_stateless_action!(IncreaseVolume, WindowActionGroup, "increase-volume");
relm4::new_stateless_action!(DecreaseVolume, WindowActionGroup, "decrease-volume");
relm4::new_stateless_action!(ResetVolume, WindowActionGroup, "reset-volume");
relm4::new_stateless_action!(IncreaseSpeed, WindowActionGroup, "increase-speed");
relm4::new_stateless_action!(DecreaseSpeed, WindowActionGroup, "decrease-speed");
relm4::new_stateless_action!(ResetSpeed, WindowActionGroup, "reset-speed");
relm4::new_stateless_action!(ToggleFullScreen, WindowActionGroup, "toggle-full-screen");
relm4::new_stateless_action!(DumpPipeline, WindowActionGroup, "dump-pipeline");
relm4::new_stateful_action!(AudioTrack, WindowActionGroup, "audio-track", String, String);

const VOLUME_DEFAULT: f64 = 1.0;
const VOLUME_MIN: f64 = 0.0;
const VOLUME_MAX: f64 = 1.0;
const VOLUME_INCREASE: f64 = 0.1;
const VOLUME_DECREASE: f64 = -VOLUME_INCREASE;

const SPEED_DEFAULT: f64 = 1.0;
const SPEED_MIN: f64 = 0.2;
const SPEED_MAX: f64 = 10.0;
const SPEED_INCREASE: f64 = 0.2;
const SPEED_DECREASE: f64 = -SPEED_INCREASE;

const DISABLE_TRACK: &str = "disable";

#[allow(deprecated)]
#[relm4::component(pub)]
impl Component for App {
    type Init = AppInit;
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = AppCmd;

    view! {
        #[local]
        app -> adw::Application {},

        #[root]
        #[name = "main_window"]
        adw::ApplicationWindow {
            #[watch]
            set_default_size: (model.preferences.data().window_size.width, model.preferences.data().window_size.height),

            #[wrap(Some)]
            set_content: main_box = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        #[watch]
                        set_title: model.window_title.as_str(),
                    },

                    pack_start = toggle_button = &gtk::ToggleButton {
                        set_icon_name: icon_names::DOCK_LEFT,
                        set_tooltip_text: Some(&fl!("toggle-playlist-visibility")),

                        set_active: model.preferences.data().show_flap,

                        connect_clicked[sender] => move |toggle_button| {
                            trace!("Toggle button clicked: is_active={}", toggle_button.is_active());
                            sender.input(AppMsg::ShowPlaylist(toggle_button.is_active()));
                        }
                    },

                    pack_end = &gtk::MenuButton {
                        set_icon_name: icon_names::MENU,
                        set_menu_model: Some(&menu_model),
                    },
                },

                #[name(toast_overlay)]
                adw::ToastOverlay {
                    #[wrap(Some)]
                    set_child: flap = &adw::Flap {
                        set_reveal_flap: model.preferences.data().show_flap,

                        connect_reveal_flap_notify[sender] => move |flap| {
                            trace!("Flap reveal notify: show={}", flap.reveals_flap());
                            sender.input(AppMsg::ShowPlaylistChanged(flap.reveals_flap()));
                        },

                        #[wrap(Some)]
                        set_flap = model.playlist_component.widget(),

                        #[wrap(Some)]
                        set_separator = &gtk::Separator{},

                        #[wrap(Some)]
                        set_content = &gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            append = playback_stack = &gtk::Stack {
                                set_vexpand: true,
                                set_hexpand: true,
                                set_vhomogeneous: false,
                                set_hhomogeneous: false,

                                #[name(playback_stack_page_view)]
                                gtk::Overlay {
                                    model.player_component.widget(),

                                    add_overlay = overlay = &gtk::Box {
                                        #[watch]
                                        set_visible: model.playlist_component.model().uris.is_empty(),
                                        add_css_class: "osd",
                                        set_vexpand: true,
                                        set_hexpand: true,

                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_hexpand: true,
                                            set_vexpand: true,
                                            set_halign: gtk::Align::Center,
                                            set_valign: gtk::Align::Center,
                                            set_spacing: 8,

                                            gtk::Label {
                                                #[watch]
                                                set_label: &fl!("drop-files-to-add"),
                                                set_css_classes: &["title-3"],
                                            },
                                        },
                                    },
                                },

                                #[name(playback_stack_page_error)]
                                adw::StatusPage {
                                    set_vexpand: true,
                                    set_hexpand: true,
                                    set_icon_name: Some(icon_names::PLAYBACK_ERROR),
                                    set_title: &fl!("he-is-dead-jim"),
                                }
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                add_css_class: "toolbar",

                                gtk::Button {
                                    set_icon_name: icon_names::ARROW_PREVIOUS_FILLED,
                                    set_tooltip_text: Some(&fl!("previous-uri")),
                                    ActionablePlus::set_stateless_action::<PrevUri>: &(),
                                },

                                #[name(play_pause_button)]
                                gtk::Button {
                                    #[watch]
                                    set_icon_name: {
                                        if model.app_state == AppState::Paused || model.app_state == AppState::Stopped {
                                            icon_names::PLAY_LARGE
                                        } else  {
                                            icon_names::PAUSE_LARGE
                                        }
                                    },
                                    set_tooltip_text: Some(&fl!("toggle-play-pause")),
                                    ActionablePlus::set_stateless_action::<TogglePlayPause>: &(),
                                },

                                gtk::Button {
                                    set_icon_name: icon_names::STEP_OVER,
                                    set_tooltip_text: Some(&fl!("next-frame", "desc")),
                                    ActionablePlus::set_stateless_action::<NextFrame>: &(),
                                },

                                gtk::Button {
                                    set_icon_name: icon_names::ARROW_NEXT_FILLED,
                                    set_tooltip_text: Some(&fl!("next-uri")),
                                    ActionablePlus::set_stateless_action::<NextUri>: &(),
                                },

                                gtk::Button {
                                    set_icon_name: icon_names::STOP_LARGE,
                                    set_tooltip_text: Some(&fl!("stop")),
                                    ActionablePlus::set_stateless_action::<Stop>: &(),
                                },

                                #[name(slider)]
                                gtk::Scale {
                                    set_hexpand: true,
                                    set_adjustment = &gtk::Adjustment {
                                        set_lower: 0.0,
                                        #[watch]
                                        set_upper: model.duration,
                                    },

                                    #[watch]
                                    #[block_signal(position_changed_handler)]
                                    set_value: model.current_position,

                                    connect_value_changed[sender] => move |scale| {
                                        sender.input(AppMsg::Seek(scale.value()));
                                    } @position_changed_handler,
                                },

                                gtk::Label {
                                    #[watch]
                                    set_markup: &format!("<span font_desc=\"monospace\">{:.0}</span>", mxl_player_components::gst::ClockTime::from_seconds(model.current_position as u64)),
                                },
                                gtk::Label {
                                    set_label: "/"
                                },
                                gtk::Label {
                                    #[watch]
                                    set_markup: &format!("<span font_desc=\"monospace\">{:.0}</span>", mxl_player_components::gst::ClockTime::from_seconds(model.duration as u64)),
                                },

                                gtk::VolumeButton {
                                    set_use_symbolic: true,

                                    #[watch]
                                    #[block_signal(volume_changed_handler)]
                                    set_value: model.volume,

                                    connect_value_changed[sender] => move |_, value| {
                                        sender.input(AppMsg::ChangeVolume(value));
                                    } @volume_changed_handler,
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    // Initialize the component.
    fn init(app_init: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let app = relm4::main_adw_application();

        let decoder_factories = gst::ElementFactory::factories_with_type(
            gst::ElementFactoryType::from_bits(
                gst::ElementFactoryType::DECODER.bits() | gst::ElementFactoryType::MEDIA_VIDEO.bits(),
            )
            .expect("Cannot create ElementFactoryType from bits"),
            gst::Rank::MARGINAL,
        );

        let mut decoder_info =
            mxl_player_components::ui::codec_ranking::CodecInfoListBuilder::new(decoder_factories).build();

        let preferences = match application::preferences::PreferencesManager::init() {
            Ok(preferences) => preferences,
            Err(error) => {
                sender.input(AppMsg::Warning(WarnMsg {
                    title: fl!("preferences-load-failed-title"),
                    warning: error.to_string(),
                    informative: Some(fl!("preferences-fallback-to-default")),
                }));
                PreferencesManager::default()
            }
        };

        for name in &preferences.data().decoder_ignore_list {
            let registry = gst::Registry::get();
            if let Some(pf) = registry.find_feature(name.as_str(), gst::ElementFactory::static_type()) {
                pf.set_rank(gst::Rank::NONE);
            } else {
                info!("Cannot change rank for decoder with name: {name}");
            }
            if let Some(info) = decoder_info.iter_mut().find(|i| i.name == *name) {
                info.enabled = false;
            }
        }

        App::set_color_scheme(&preferences.data().color_scheme);

        let playlist_component = PlaylistComponentModel::builder()
            .launch(PlaylistComponentInit { uris: app_init.uris })
            .forward(sender.command_sender(), |output| match output {
                PlaylistComponentOutput::PlaylistChanged(_) => AppCmd::PlaylistChanged,
                PlaylistComponentOutput::SwitchUri(x) => AppCmd::PlaylistSwitchUri(x),
                PlaylistComponentOutput::EndOfPlaylist => AppCmd::PlaylistEndOfPlaylist,
                PlaylistComponentOutput::StateChanged(state) => AppCmd::PlaylistStateChanged(state),
                PlaylistComponentOutput::FileChooserRequest => AppCmd::PlaylistFileChooserRequest,
            });

        let preferences_dialog = PreferencesComponentModel::builder()
            .transient_for(&root)
            .launch(PreferencesComponentInit {
                color_scheme: preferences.data().color_scheme.clone(),
                auto_play: preferences.data().auto_play,
                decoder_info,
            })
            .forward(sender.command_sender(), |output| match output {
                PreferencesComponentOutput::ColorScheme(x) => AppCmd::PreferencesSetColorScheme(x),
                PreferencesComponentOutput::AutoPlay(x) => AppCmd::PreferencesSetAutoPlay(x),
                PreferencesComponentOutput::DecoderRank(name, rank) => AppCmd::PreferencesSetDecoderRank(name, rank),
            });

        let player_component = {
            PlayerComponentModel::builder()
                .launch(PlayerComponentInit {
                    seek_accurate: false,
                    show_seeking_overlay: false,
                    compositor: app_init.compositor,
                    draw_callback: Box::new(|_, _| {}),
                    drag_gesture: None,
                    motion_tracker: None,
                })
                .forward(sender.command_sender(), |output| match output {
                    PlayerComponentOutput::PlayerInitialized(x) => AppCmd::PlayerInitialized(x),
                    PlayerComponentOutput::MediaInfoUpdated(x) => AppCmd::PlayerMediaInfoUpdated(x),
                    PlayerComponentOutput::DurationChanged(x) => AppCmd::PlayerDurationChanged(x),
                    PlayerComponentOutput::PositionUpdated(x) => AppCmd::PlayerPositionUpdated(x),
                    PlayerComponentOutput::SeekDone => AppCmd::PlayerSeekDone,
                    PlayerComponentOutput::EndOfStream(x) => AppCmd::PlayerEndOfStream(x),
                    PlayerComponentOutput::StateChanged(x, y) => AppCmd::PlayerStateChanged(x, y),
                    PlayerComponentOutput::VolumeChanged(x) => AppCmd::PlayerVolumeChanged(x),
                    PlayerComponentOutput::SpeedChanged(x) => AppCmd::PlayerSpeedChanged(x),
                    PlayerComponentOutput::AudioVideoOffsetChanged(x) => AppCmd::PlayerAudioVideoOffsetChanged(x),
                    PlayerComponentOutput::SubtitleVideoOffsetChanged(x) => AppCmd::PlayerSubtitleVideoOffsetChanged(x),
                    PlayerComponentOutput::Warning(x) => AppCmd::PlayerWarning(x),
                    PlayerComponentOutput::Error(x) => AppCmd::PlayerError(x),
                })
        };

        let mut model = App {
            request_exit: false,
            ready_to_exit: Arc::new(Mutex::new(false)),
            preferences,
            window_title: get_window_title(None),
            current_position: 0.0,
            duration: 0.0,
            volume: VOLUME_DEFAULT,
            speed: SPEED_DEFAULT,
            app_state: AppState::Stopped,
            auto_start_done: false,
            reload_player_on_stopped: false,
            audio_track_items: vec![],
            playlist_component,
            preferences_dialog,
            player_component,
            message_dialog: {
                MessageDialog::builder()
                    .transient_for(&root)
                    .launch(())
                    .forward(sender.command_sender(), |output| match output {
                        MessageDialogOutput::CreateReport => AppCmd::MessageDialogCreateReport,
                        MessageDialogOutput::Quit => AppCmd::MessageDialogQuitApp,
                    })
            },
            problem_report_dialog: ProblemReportDialog::builder()
                .launch(ProblemReportDialogInit {
                    app_name: about::APP_NAME,
                    binary_name: about::BINARY_NAME,
                })
                .forward(sender.input_sender(), |msg| match msg {
                    ProblemReportDialogOutput::Closed => AppMsg::ProblemReportDialogClosed,
                }),
            create_report_dialog: CreateReportDialog::builder()
                .launch(CreateReportDialogInit {
                    app_name: about::APP_NAME,
                    binary_name: about::BINARY_NAME,
                })
                .detach(),
            file_open_dialog: {
                let mut mime_types: Vec<_> = mxl_player_components::gst::ElementFactory::factories_with_type(
                    mxl_player_components::gst::ElementFactoryType::DEMUXER,
                    mxl_player_components::gst::Rank::MARGINAL,
                )
                .iter()
                .flat_map(|factory| {
                    let mime_types: Vec<_> = factory
                        .static_pad_templates()
                        .iter()
                        .filter_map(|pad| {
                            if pad.direction() == mxl_player_components::gst::PadDirection::Sink {
                                let mime_types: Vec<_> = pad.caps().iter().map(|cap| cap.name().to_string()).collect();
                                Some(mime_types)
                            } else {
                                None
                            }
                        })
                        .flatten()
                        .collect();
                    mime_types
                })
                .collect();
                mime_types.dedup();
                trace!("Supported mime types: {mime_types:?}");

                OpenDialogMulti::builder()
                    .transient_for_native(&root)
                    .launch(OpenDialogSettings {
                        create_folders: false,
                        folder_mode: false,
                        is_modal: true,
                        filters: vec![
                            {
                                let filter = gtk::FileFilter::new();
                                filter.set_name(Some(&fl!("media-files")));
                                for mime_type in mime_types {
                                    filter.add_mime_type(mime_type.as_str());
                                }
                                filter
                            },
                            {
                                let filter = gtk::FileFilter::new();
                                filter.set_name(Some(&fl!("all-files")));
                                filter.add_pattern("*");
                                filter
                            },
                        ],
                        ..Default::default()
                    })
                    .forward(sender.input_sender(), |response| match response {
                        OpenDialogResponse::Accept(path) => AppMsg::FileChooserAccept(path),
                        OpenDialogResponse::Cancel => AppMsg::FileChooserIgnore,
                    })
            },
            video_offsets_dialog: {
                VideoOffsetsComponentModel::builder()
                    .transient_for(&root)
                    .launch(VideoOffsetsComponentInit {
                        audio_video_offset: 10,
                        subtitle_video_offset: 0,
                    })
                    .forward(sender.command_sender(), |output| match output {
                        VideoOffsetsComponentOutput::SetAudioVideoOffset(x) => {
                            AppCmd::VideoOffsetsDialogAudioVideoOffsetChanged(x)
                        }
                        VideoOffsetsComponentOutput::SetSubtitleVideoOffset(x) => {
                            AppCmd::VideoOffsetsDialogSubtitleVideoOffsetChanged(x)
                        }
                    })
            },
            about_dialog: {
                adw::AboutWindow::builder()
                    .application_name(about::APP_NAME)
                    .application_icon(about::APP_ID)
                    .transient_for(&root)
                    .hide_on_close(true)
                    .modal(true)
                    .license_type(gtk::License::Apache20)
                    .copyright(about::COPYRIGHT)
                    .version(about::VERSION)
                    .website("https://".to_owned() + about::ORGANIZATION_DOMAIN)
                    .build()
            },
            third_party_licenses_dialog: {
                ThirdPartyLicensesComponentModel::builder()
                    .transient_for(&root)
                    .launch(())
            },
            update_actions: Vec::new(),
            audio_track_menu: gio::Menu::new(),
        };

        {
            let mut action_group = RelmActionGroup::<AppActionGroup>::new();
            action_group.add_action(RelmAction::<Quit>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::Quit)
            )));
            action_group.register_for_main_application();
            app.set_accelerators_for_action::<Quit>(&actions::accelerators(Accelerators::Quit));

            // Window specific hotkeys:
            app.set_accelerators_for_action::<TogglePlayPause>(&actions::accelerators(Accelerators::TogglePlayPause));
            app.set_accelerators_for_action::<Preferences>(&actions::accelerators(Accelerators::Preferences));
            app.set_accelerators_for_action::<VideoOffsets>(&actions::accelerators(Accelerators::VideoOffsets));
            app.set_accelerators_for_action::<FileChooser>(&actions::accelerators(Accelerators::FileChooser));
            app.set_accelerators_for_action::<NextFrame>(&actions::accelerators(Accelerators::NextFrame));
            app.set_accelerators_for_action::<NextUri>(&actions::accelerators(Accelerators::Next));
            app.set_accelerators_for_action::<PrevUri>(&actions::accelerators(Accelerators::Previous));
            app.set_accelerators_for_action::<IncreaseVolume>(&actions::accelerators(Accelerators::IncreaseVolume));
            app.set_accelerators_for_action::<DecreaseVolume>(&actions::accelerators(Accelerators::DecreaseVolume));
            app.set_accelerators_for_action::<IncreaseSpeed>(&actions::accelerators(Accelerators::IncreaseSpeed));
            app.set_accelerators_for_action::<DecreaseSpeed>(&actions::accelerators(Accelerators::DecreaseSpeed));
            app.set_accelerators_for_action::<ResetSpeed>(&actions::accelerators(Accelerators::ResetSpeed));
            app.set_accelerators_for_action::<DumpPipeline>(&actions::accelerators(Accelerators::DumpPipeline));
            app.set_accelerators_for_action::<ToggleFullScreen>(&actions::accelerators(Accelerators::FullScreen));
            app.set_accelerators_for_action::<TogglePlaylistVisibility>(&actions::accelerators(
                Accelerators::TogglePlaylistVisibility,
            ));
        }

        let menu_model = gtk::gio::Menu::new();
        menu_model.append(Some(&fl!("add-files")), Some(&FileChooser::action_name()));
        menu_model.append(Some(&fl!("toggle-play-pause")), Some(&TogglePlayPause::action_name()));
        menu_model.append(Some(&fl!("stop")), Some(&Stop::action_name()));
        menu_model.append(Some(&fl!("next-frame")), Some(&NextFrame::action_name()));
        menu_model.append(Some(&fl!("previous-uri")), Some(&PrevUri::action_name()));
        menu_model.append(Some(&fl!("next-uri")), Some(&NextUri::action_name()));
        menu_model.append(
            Some(&fl!("toggle-playlist-visibility")),
            Some(&TogglePlaylistVisibility::action_name()),
        );
        menu_model.append_submenu(Some(&fl!("volume")), &{
            let m = gio::Menu::new();
            m.append(Some(&fl!("increase")), Some(&IncreaseVolume::action_name()));
            m.append(Some(&fl!("reset")), Some(&ResetVolume::action_name()));
            m.append(Some(&fl!("decrease")), Some(&DecreaseVolume::action_name()));
            m
        });
        menu_model.append_submenu(Some(&fl!("audio-track")), &model.audio_track_menu);
        menu_model.append_submenu(Some(&fl!("speed")), &{
            let m = gio::Menu::new();
            m.append(Some(&fl!("increase")), Some(&IncreaseSpeed::action_name()));
            m.append(Some(&fl!("reset")), Some(&ResetSpeed::action_name()));
            m.append(Some(&fl!("decrease")), Some(&DecreaseSpeed::action_name()));
            m
        });
        menu_model.append(Some(&fl!("toggle-full-screen")), Some(&ToggleFullScreen::action_name()));
        if cfg!(debug_assertions) {
            menu_model.append(Some(&fl!("dump-pipeline")), Some(&DumpPipeline::action_name()));
        }
        menu_model.append_section(None, &{
            let m = gio::Menu::new();
            m.append(Some(&fl!("video-offsets")), Some(&VideoOffsets::action_name()));
            m.append(Some(&fl!("preferences-dialog")), Some(&Preferences::action_name()));
            m.append(Some(&fl!("create-report")), Some(&CreateReport::action_name()));
            m.append(Some(&fl!("about")), Some(&About::action_name()));
            m.append(
                Some(&fl!("third-party-licenses")),
                Some(&ThirdPartyLicenses::action_name()),
            );
            m.append(Some(&fl!("quit")), Some(&Quit::action_name()));
            m
        });

        // Generate the widgets based on the view! macro here
        let widgets = view_output!();

        {
            let mut action_group = RelmActionGroup::<WindowActionGroup>::new();
            action_group.add_action(RelmAction::<About>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ShowAboutDialog)
            )));
            action_group.add_action(RelmAction::<CreateReport>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::CreateReportDialogOpen)
            )));
            action_group.add_action(RelmAction::<ThirdPartyLicenses>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ShowThirdPartyLicensesDialog)
            )));
            action_group.add_action(RelmAction::<Preferences>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ShowPreferencesDialog)
            )));
            action_group.add_action(RelmAction::<VideoOffsets>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ShowVideoOffsetsDialog)
            )));
            action_group.add_action(RelmAction::<FileChooser>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::FileChooserRequest)
            )));
            action_group.add_action(RelmAction::<TogglePlaylistVisibility>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::TogglePlaylistVisibility)
            )));
            {
                let action = RelmAction::<TogglePlayPause>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::TogglePlayPause)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Error);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<NextFrame>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::NextFrame)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state == AppState::Paused || app_state == AppState::Playing);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<Stop>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::Stop)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<NextUri>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::Next)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(
                            app_state == AppState::Paused
                                || app_state == AppState::Buffering
                                || app_state == AppState::Playing
                                || app_state == AppState::Error,
                        );
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<PrevUri>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::Previous)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(
                            app_state == AppState::Paused
                                || app_state == AppState::Buffering
                                || app_state == AppState::Playing
                                || app_state == AppState::Error,
                        );
                    }
                )));
                action_group.add_action(action);
            }
            action_group.add_action(RelmAction::<IncreaseVolume>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::IncreaseVolume)
            )));
            action_group.add_action(RelmAction::<DecreaseVolume>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::DecreaseVolume)
            )));
            action_group.add_action(RelmAction::<ResetVolume>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ResetVolume)
            )));
            {
                let action = RelmAction::<IncreaseSpeed>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::IncreaseSpeed)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<DecreaseSpeed>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::DecreaseSpeed)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<ResetSpeed>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::ResetSpeed)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            action_group.add_action(RelmAction::<ToggleFullScreen>::new_stateless(clone!(
                #[strong]
                sender,
                move |_| sender.input(AppMsg::ToggleFullScreen)
            )));
            {
                let action = RelmAction::<DumpPipeline>::new_stateless(clone!(
                    #[strong]
                    sender,
                    move |_| sender.input(AppMsg::DumpPipeline)
                ));
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            {
                let action = RelmAction::<AudioTrack>::new_stateful_with_target_value(
                    &format!("{}::0", AudioTrack::action_name()),
                    clone!(
                        #[strong]
                        sender,
                        move |_, _state, target| sender.input(AppMsg::SwitchAudioTrack(target))
                    ),
                );
                model.update_actions.push(Box::new(clone!(
                    #[strong(rename_to = gio_action)]
                    action.gio_action(),
                    move |app_state| {
                        gio_action.set_enabled(app_state != AppState::Stopped);
                    }
                )));
                action_group.add_action(action);
            }
            action_group.register_for_widget(&widgets.main_window);
        }

        model.set_audio_track_menu(None);

        widgets
            .playback_stack_page_view
            .add_controller(PlaylistComponentModel::new_drop_target(
                model.playlist_component.sender().clone(),
            ));

        widgets.main_window.connect_close_request(clone!(
            #[weak(rename_to = ready_to_exit)]
            model.ready_to_exit,
            #[upgrade_or]
            gtk::glib::Propagation::Proceed,
            move |_| {
                if !(*ready_to_exit.lock().unwrap()) {
                    sender.input(AppMsg::Quit);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed
                }
            }
        ));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            AppMsg::PlayerInitialized => match mxl_investigator::proc_dir::any_panic() {
                Ok(any_panic) => {
                    if any_panic {
                        sender.input(AppMsg::ProblemReportDialogOpen);
                    } else {
                        sender.input(AppMsg::DoAutoStart);
                    }
                }
                Err(error) => {
                    sender.input(AppMsg::Warning(WarnMsg {
                        title: fl!("get-failed-procs-failed-title"),
                        warning: error.to_string(),
                        informative: None,
                    }));
                }
            },
            AppMsg::TogglePlaylistVisibility => {
                widgets.flap.set_reveal_flap(!self.preferences.data().show_flap);
            }
            AppMsg::ShowPlaylist(show) => {
                trace!("Show Flap msg: show={show}");
                widgets.flap.set_reveal_flap(show);
            }
            AppMsg::ShowPlaylistChanged(show) => {
                trace!("Playlist visibility changed msg: show={show}");
                self.preferences.data_mut().show_flap = show;
                widgets.toggle_button.set_active(show);
            }
            AppMsg::TogglePlayPause => {
                debug!("Play/pause");
                match self.app_state {
                    AppState::Stopped => {
                        self.playlist_component
                            .sender()
                            .send(PlaylistComponentInput::Start)
                            .unwrap_or_default();
                    }
                    AppState::Playing => {
                        self.player_component
                            .sender()
                            .send(PlayerComponentInput::ChangeState(PlaybackState::Paused))
                            .unwrap_or_default();
                    }
                    AppState::Paused => {
                        self.player_component
                            .sender()
                            .send(PlayerComponentInput::ChangeState(PlaybackState::Playing))
                            .unwrap_or_default();
                    }
                    AppState::Buffering => {
                        self.player_component
                            .sender()
                            .send(PlayerComponentInput::ChangeState(PlaybackState::Paused))
                            .unwrap_or_default();
                    }
                    AppState::Next => (),
                    AppState::Previous => (),
                    AppState::Error => (),
                }
            }
            AppMsg::NextFrame => {
                // Stepping to the next frame is only allowed while in pause:
                if self.app_state != AppState::Paused {
                    sender.input(AppMsg::TogglePlayPause);
                } else {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::NextFrame)
                        .unwrap_or_default();
                }
            }
            AppMsg::Stop => {
                self.playlist_component
                    .sender()
                    .send(PlaylistComponentInput::Stop)
                    .unwrap_or_default();
            }
            AppMsg::Stopped => {
                self.set_audio_track_menu(None);
                if self.request_exit {
                    sender.input(AppMsg::Quit);
                }
                if self.reload_player_on_stopped {
                    self.reload_player_on_stopped = false;
                    self.player_component.sender().emit(PlayerComponentInput::ReloadPlayer);
                }
            }
            AppMsg::SwitchUri(uri) => {
                sender.input(AppMsg::ResetSpeed);
                self.player_component
                    .sender()
                    .send(PlayerComponentInput::UpdateUri(uri))
                    .unwrap_or_default();
                self.player_component
                    .sender()
                    .send(PlayerComponentInput::ChangeState(PlaybackState::Playing))
                    .unwrap_or_default();
            }
            AppMsg::Previous => {
                trace!("Switch to next previous");
                self.app_state = AppState::Previous;
                self.playlist_component
                    .sender()
                    .send(PlaylistComponentInput::Previous)
                    .unwrap_or_default();
            }
            AppMsg::Next => {
                trace!("Switch to next file");
                self.app_state = AppState::Next;
                self.playlist_component
                    .sender()
                    .send(PlaylistComponentInput::Next)
                    .unwrap_or_default();
            }
            AppMsg::Seek(to) => match self.app_state {
                AppState::Stopped => (),
                _ => {
                    self.current_position = to;
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::Seek(to))
                        .unwrap_or_default();
                }
            },
            AppMsg::SwitchAudioTrack(stream_index) => {
                trace!("Switch to audio stream {stream_index}");
                if stream_index == DISABLE_TRACK {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SwitchAudioTrack(Track::Disable))
                        .unwrap_or_default();
                } else {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SwitchAudioTrack(Track::Stream(
                            stream_index.parse().unwrap_or_default(),
                        )))
                        .unwrap_or_default();
                }
            }
            AppMsg::IncreaseVolume => {
                sender.input(AppMsg::ChangeVolume(App::clamp_volume(self.volume + VOLUME_INCREASE)));
            }
            AppMsg::DecreaseVolume => {
                sender.input(AppMsg::ChangeVolume(App::clamp_volume(self.volume + VOLUME_DECREASE)));
            }
            AppMsg::ResetVolume => {
                sender.input(AppMsg::ChangeVolume(VOLUME_DEFAULT));
            }
            AppMsg::SetVolume(vol) => {
                trace!("volume was set to {vol}");
                self.volume = vol;
            }
            AppMsg::ChangeVolume(vol) => {
                if self.volume != vol {
                    trace!("change volume to {vol}");
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SetVolume(vol))
                        .unwrap_or_default();
                }
            }
            AppMsg::IncreaseSpeed => {
                sender.input(AppMsg::ChangeSpeed(App::clamp_speed(self.speed + SPEED_INCREASE)));
            }
            AppMsg::DecreaseSpeed => {
                sender.input(AppMsg::ChangeSpeed(App::clamp_speed(self.speed + SPEED_DECREASE)));
            }
            AppMsg::ResetSpeed => {
                sender.input(AppMsg::ChangeSpeed(SPEED_DEFAULT));
            }
            AppMsg::SetSpeed(speed) => {
                trace!("speed was set to {speed}");
                self.speed = speed;
            }
            AppMsg::ChangeSpeed(speed) => {
                if self.speed != speed {
                    trace!("change speed to {speed}");
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SetSpeed(speed))
                        .unwrap_or_default();
                }
            }
            AppMsg::SetAudioVideoOffset(offset) => {
                if self.video_offsets_dialog.model().audio_video_offset() != offset {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SetAudioVideoOffset(offset))
                        .unwrap_or_default();
                }
            }
            AppMsg::SetSubtitleVideoOffset(offset) => {
                if self.video_offsets_dialog.model().subtitle_video_offset() != offset {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::SetSubtitleVideoOffset(offset))
                        .unwrap_or_default();
                }
            }
            AppMsg::PlayerMediaInfoUpdated(media_info) => self.set_audio_track_menu(Some(&media_info)),
            AppMsg::ToggleFullScreen => {
                if widgets.main_window.is_fullscreen() {
                    trace!("Leave full screen");
                    widgets.main_window.unfullscreen();
                } else {
                    trace!("Enter full screen");
                    widgets.main_window.fullscreen();
                }
            }
            AppMsg::DumpPipeline => {
                debug!("Dump pipeline");
                self.dump_pipeline();
                widgets.toast_overlay.add_toast(toast(fl!("dumped-pipeline"), 1));
            }
            AppMsg::FileChooserRequest => self.file_open_dialog.emit(OpenDialogMsg::Open),
            AppMsg::FileChooserAccept(paths) => self.playlist_component.emit(PlaylistComponentInput::Add(paths)),
            AppMsg::FileChooserIgnore => (),
            AppMsg::ShowVideoOffsetsDialog => {
                let video_offsets_dialog = self.video_offsets_dialog.widget();
                video_offsets_dialog.present();
            }
            AppMsg::ShowAboutDialog => {
                self.about_dialog.present();
            }
            AppMsg::ShowPreferencesDialog => {
                let preferences_dialog = self.preferences_dialog.widget();
                preferences_dialog.present();
            }
            AppMsg::ShowThirdPartyLicensesDialog => {
                let third_party_licenses_dialog = self.third_party_licenses_dialog.widget();
                third_party_licenses_dialog.present();
            }
            AppMsg::Quit => {
                if self.app_state != AppState::Stopped {
                    self.request_exit = true;
                    sender.input(AppMsg::Stop);
                } else {
                    sender.input(AppMsg::SaveWindowState {
                        width: widgets.main_window.width(),
                        height: widgets.main_window.height(),
                    });
                    {
                        let mut rte = self.ready_to_exit.lock().unwrap();
                        *rte = true;
                    }
                    widgets.main_window.close();
                }
            }
            AppMsg::SaveWindowState { width, height } => {
                self.preferences.data_mut().window_size = WindowSize { width, height };
                match self.preferences.save().with_context(|| "Cannot save application state") {
                    Ok(_) => trace!("saved application state"),
                    Err(error) => {
                        error!("{error:?}");
                        self.message_dialog.emit(MessageDialogInput::Message(
                            MessageDialogType::Error,
                            None,
                            format!("{error:?}"),
                        ));
                    }
                };
            }
            AppMsg::ShowPlaybackView => {
                widgets
                    .playback_stack
                    .set_visible_child(&widgets.playback_stack_page_view);
            }
            AppMsg::PlaybackError(error) => {
                widgets
                    .playback_stack_page_error
                    .set_description(Some(error.to_string().as_str()));
                widgets
                    .playback_stack
                    .set_visible_child(&widgets.playback_stack_page_error);
            }
            AppMsg::Warning(warn_msg) => {
                let msg = if let Some(informative) = warn_msg.informative {
                    format!("'{}' – {}", warn_msg.warning, informative)
                } else {
                    warn_msg.warning.clone()
                };
                warn!("{}: '{}'", warn_msg.title, msg);
                self.message_dialog.emit(MessageDialogInput::Message(
                    MessageDialogType::Warning,
                    Some(warn_msg.title),
                    msg,
                ));
            }
            AppMsg::ProblemReportDialogOpen => {
                self.dump_pipeline();
                self.problem_report_dialog.emit(ProblemReportDialogInput::Present(
                    root.upcast_ref::<gtk::Widget>().clone(),
                ));
            }
            AppMsg::ProblemReportDialogClosed => {
                sender.input(AppMsg::DoAutoStart);
            }
            AppMsg::CreateReportDialogOpen => {
                self.dump_pipeline();
                self.create_report_dialog.emit(CreateReportDialogInput::Present(
                    root.upcast_ref::<gtk::Widget>().clone(),
                ));
            }
            AppMsg::DoAutoStart => {
                if !self.auto_start_done {
                    self.auto_start_done = true;
                    if self.preferences.data().auto_play && !self.playlist_component.model().uris.is_empty() {
                        sender.input(AppMsg::TogglePlayPause);
                    }
                }
            }
        }
        self.update_actions();
        self.update_view(widgets, sender)
    }

    fn update_cmd(&mut self, msg: Self::CommandOutput, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AppCmd::PlayerInitialized(error) => {
                if let Some(error) = error {
                    error!("{error:?}");
                    self.message_dialog.emit(MessageDialogInput::Message(
                        MessageDialogType::Fatal,
                        None,
                        format!("{error:?}"),
                    ));
                } else {
                    sender.input(AppMsg::PlayerInitialized);
                }
            }
            AppCmd::PlayerMediaInfoUpdated(media_info) => {
                self.window_title = get_window_title(Some(media_info.clone()));
                sender.input(AppMsg::PlayerMediaInfoUpdated(media_info));
            }
            AppCmd::PlayerEndOfStream(a) => {
                debug!("player end of stream : {a}");
                sender.input(AppMsg::Next)
            }
            AppCmd::PlayerDurationChanged(duration) => {
                self.duration = duration;
            }
            AppCmd::PlayerPositionUpdated(pos) => {
                self.current_position = pos;
                // debug!("player position updated {pos}");
            }
            AppCmd::PlayerSeekDone => {
                debug!("player seek done");
            }
            AppCmd::PlayerStateChanged(old_state, new_state) => {
                debug!("playback state changed from {old_state:?} to {new_state:?}");
                if let Some(old_state) = old_state {
                    match old_state {
                        PlaybackState::Stopped => (),
                        PlaybackState::Playing => (),
                        PlaybackState::Paused => (),
                        PlaybackState::Buffering => (),
                        PlaybackState::Error => sender.input(AppMsg::ShowPlaybackView),
                    }
                }
                match new_state {
                    PlaybackState::Stopped => {
                        self.app_state = AppState::Stopped;
                        self.window_title = get_window_title(None);
                        self.duration = 0.0;
                        self.current_position = 0.0;
                        self.playlist_component
                            .sender()
                            .send(PlaylistComponentInput::PlayerStopped)
                            .unwrap_or_default();
                    }
                    PlaybackState::Playing => {
                        self.app_state = AppState::Playing;
                        self.playlist_component
                            .sender()
                            .send(PlaylistComponentInput::PlayerPlaying)
                            .unwrap_or_default();
                    }
                    PlaybackState::Paused => self.app_state = AppState::Paused,
                    PlaybackState::Buffering => self.app_state = AppState::Buffering,
                    PlaybackState::Error => self.app_state = AppState::Error,
                }
            }
            AppCmd::PlayerVolumeChanged(vol) => sender.input(AppMsg::SetVolume(vol)),
            AppCmd::PlayerSpeedChanged(speed) => sender.input(AppMsg::SetSpeed(speed)),
            AppCmd::PlayerAudioVideoOffsetChanged(offset) => {
                sender.input(AppMsg::SetAudioVideoOffset(offset));
                self.video_offsets_dialog
                    .sender()
                    .send(VideoOffsetsComponentInput::SetAudioVideoOffset(offset))
                    .unwrap_or_default();
            }
            AppCmd::PlayerSubtitleVideoOffsetChanged(offset) => {
                self.video_offsets_dialog
                    .sender()
                    .send(VideoOffsetsComponentInput::SetSubtitleVideoOffset(offset))
                    .unwrap_or_default();
            }
            AppCmd::PlayerWarning(error) => {
                warn!("Internal player warning: {error:?}");
            }
            AppCmd::PlayerError(error) => {
                error!("Internal player error: {error:?}");
                sender.input(AppMsg::PlaybackError(error));
            }
            AppCmd::PlaylistChanged => (),
            AppCmd::PlaylistSwitchUri(uri) => sender.input(AppMsg::SwitchUri(uri)),
            AppCmd::PlaylistEndOfPlaylist => sender.input(AppMsg::Stop),
            AppCmd::PlaylistStateChanged(state) => match state {
                PlaylistState::Playing => (),
                PlaylistState::Stopping => {
                    self.player_component
                        .sender()
                        .send(PlayerComponentInput::ChangeState(PlaybackState::Stopped))
                        .unwrap_or_default();
                }
                PlaylistState::Stopped => {
                    sender.input(AppMsg::Stopped);
                }
            },
            AppCmd::PlaylistFileChooserRequest => sender.input(AppMsg::FileChooserRequest),
            AppCmd::PreferencesSetColorScheme(color_scheme) => {
                App::set_color_scheme(&color_scheme);
                self.preferences.data_mut().color_scheme = color_scheme;
            }
            AppCmd::PreferencesSetAutoPlay(value) => {
                self.preferences.data_mut().auto_play = value;
            }
            AppCmd::PreferencesSetDecoderRank(name, rank) => {
                warn!("Encoder enabled: {name} = {rank:?}");
                // Change the rank in the GStreamer registry:
                let registry = gst::Registry::get();
                if let Some(pf) = registry.find_feature(name.as_str(), gst::ElementFactory::static_type()) {
                    if pf.rank() != rank {
                        pf.set_rank(rank);
                        sender.input_sender().emit(AppMsg::Stop);
                        self.reload_player_on_stopped = true;
                        self.playlist_component
                            .sender()
                            .emit(PlaylistComponentInput::FetchMetadata);
                    }
                } else {
                    info!("Cannot change rank for decoder with name: {name}");
                }
                // Add or remove the decoder from the preferences decoder ignore list:
                if rank < gst::Rank::MARGINAL {
                    let data = self.preferences.data_mut();
                    if !data.decoder_ignore_list.contains(&name) {
                        data.decoder_ignore_list.push(name);
                    }
                } else {
                    let data = self.preferences.data_mut();
                    if let Some(index) = data.decoder_ignore_list.iter().position(|i| *i == name) {
                        data.decoder_ignore_list.swap_remove(index);
                    }
                }
            }
            AppCmd::VideoOffsetsDialogAudioVideoOffsetChanged(offset) => {
                sender.input(AppMsg::SetAudioVideoOffset(offset))
            }
            AppCmd::VideoOffsetsDialogSubtitleVideoOffsetChanged(offset) => {
                sender.input(AppMsg::SetSubtitleVideoOffset(offset))
            }
            AppCmd::MessageDialogCreateReport => sender.input(AppMsg::CreateReportDialogOpen),
            AppCmd::MessageDialogQuitApp => sender.input(AppMsg::Quit),
        }
        self.update_actions();
    }
}

impl App {
    fn set_color_scheme(color_scheme: &ColorScheme) {
        match color_scheme {
            ColorScheme::Default => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default),
            ColorScheme::Dark => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark),
            ColorScheme::Light => adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight),
        }
    }

    fn clamp_volume(volume: f64) -> f64 {
        volume.clamp(VOLUME_MIN, VOLUME_MAX)
    }

    fn clamp_speed(speed: f64) -> f64 {
        speed.clamp(SPEED_MIN, SPEED_MAX)
    }

    fn update_actions(&self) {
        for update_action in &self.update_actions {
            update_action(self.app_state);
        }
    }

    fn set_audio_track_menu(&mut self, info: Option<&mxl_player_components::gst_play::PlayMediaInfo>) {
        let new_track_items = if let Some(info) = info {
            info.audio_streams()
                .iter()
                .enumerate()
                .map(|(index, audio_stream)| {
                    let l = if let Some(l) = audio_stream.language() {
                        fl!(
                            "audio-stream-with-language",
                            channels = audio_stream.channels(),
                            sample_rate = audio_stream.sample_rate(),
                            language = l.to_string()
                        )
                        .to_owned()
                    } else {
                        fl!(
                            "audio-stream",
                            channels = audio_stream.channels(),
                            sample_rate = audio_stream.sample_rate()
                        )
                        .to_owned()
                    };
                    (index, l)
                })
                .collect()
        } else {
            vec![]
        };

        if new_track_items != self.audio_track_items {
            let audio_menu = gio::Menu::new();

            audio_menu.append_item(&gio::MenuItem::new(
                Some(&fl!("disable")),
                Some(&format!("{}::{}", AudioTrack::action_name(), DISABLE_TRACK)),
            ));

            for (index, label) in &new_track_items {
                audio_menu.append_item(&gio::MenuItem::new(
                    Some(label),
                    Some(&format!("{}::{}", AudioTrack::action_name(), index)),
                ));
            }

            self.audio_track_menu.remove_all();
            self.audio_track_menu.append_section(None, &audio_menu);

            self.audio_track_items = new_track_items;
        }
    }

    fn dump_pipeline(&self) {
        self.player_component
            .sender()
            .send(PlayerComponentInput::DumpPipeline(
                chrono::Local::now().format("mxl_player_%Y-%m-%d_%H_%M_%S").to_string(),
            ))
            .unwrap_or_default();
    }
}

fn toast<T: ToString>(title: T, timeout: u32) -> Toast {
    Toast::builder()
        .title(title.to_string().as_str())
        .timeout(timeout)
        .build()
}

fn get_window_title(media_info: Option<PlayMediaInfo>) -> String {
    if let Some(info) = media_info {
        let title = if let Some(title) = info.title() {
            title.to_string()
        } else {
            info.uri().to_string()
        };
        return format!("{} - {title}", about::APP_NAME);
    }
    about::APP_NAME.to_owned()
}
