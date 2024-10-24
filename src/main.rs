// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// SPDX-FileCopyrightText: 2024 X-Software GmbH <opensource@x-software.com>

use anyhow::Result;
use clap::Parser;
use log::*;
use mxl_player_components::glib_helpers;
use mxl_relm4_components::relm4::{self, gtk::gio, gtk::glib, prelude::*};
use std::env;

mod about;
mod app;
mod application;
mod commandline;
mod localization;
mod ui;

use crate::app::{App, AppInit};

fn run() -> Result<()> {
    // Note: The command line must be parsed before starting the logger!
    // parse() can terminate/exit the software, e.g. if the argument "--help" is given.
    // In this case, we cannot remove the proc directory by calling mxl_investigator::proc_dir::remove_proc_dir()
    let args = commandline::CliArgs::parse();

    mxl_base::logging::Builder::new()
        .level_for("html5ever", log::LevelFilter::Warn) // dependency of html2pango
        .build(mxl_investigator::proc_dir::proc_dir())?;

    if let Some(archive_file_path) = &args.export_bug_report {
        return mxl_investigator::proc_dir::failed_dir_archive_and_remove(archive_file_path);
    }

    if mxl_investigator::proc_dir::failed_dir_any_panic()? {
        warn!("Previous runs of the application were not properly terminated. Please start the application with --help to find out how to export a bug report.");
    }

    glib_helpers::init_logging();
    glib::set_program_name(Some(about::APP_NAME));
    gio::resources_register_include!("resources.gresource").expect("Failed to register resources");

    mxl_relm4_components::init()?;
    mxl_player_components::init(
        mxl_investigator::proc_dir::proc_dir(),
        mxl_base::misc::project_dirs().cache_dir(),
    )?;

    if args.update_registry {
        gst::Registry::update()?;
    }

    debug!("Files to open: {:?}", args.uris);

    const RELM_DEFAULT_THREADS: usize = 4;
    let thread_count =
        std::thread::available_parallelism().map_or(RELM_DEFAULT_THREADS, |v| v.get().min(RELM_DEFAULT_THREADS));
    debug!("Set the number of RELM4 threads to {thread_count}");
    relm4::RELM_THREADS.set(thread_count).map_or_else(
        |count| {
            error!("Cannot set the number of REALM_THREADS to '{}'", count);
        },
        |_| (),
    );
    relm4_icons::initialize_icons();

    let display = gtk::gdk::Display::default().expect("Cannot get the GDK default display");
    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_resource_path(about::ICONS_RESOURCES_PATH);
    gtk::Window::set_default_icon_name(about::APP_ID);

    let adw_app = adw::Application::new(Some(about::APP_ID), gio::ApplicationFlags::default());
    let app = RelmApp::from_app(adw_app);
    app.with_args(vec![]).run::<App>(AppInit {
        compositor: None,
        uris: args.uris.clone(),
    });
    Ok(())
}

fn main() -> std::process::ExitCode {
    mxl_base::init(
        about::QUALIFIER,
        about::ORGANIZATION_NAME,
        about::APP_NAME,
        about::BINARY_NAME,
        about::VERSION,
    );
    mxl_investigator::init(mxl_base::misc::project_dirs().data_local_dir().to_path_buf());
    mxl_investigator::proc_dir::setup_panic();
    mxl_investigator::proc_dir::proc_dir_archive_set_callback(|| {
        mxl_investigator::misc::create_sysinfo_dump();
        {
            // gst-inspect to get the gst plugin load logs
            let current_path = mxl_investigator::proc_dir::proc_dir();
            let registry_file_name = current_path.join("registry.bin").to_str().unwrap().to_owned();
            let mut command = std::process::Command::new("gst-inspect-1.0");
            let gst_bin_path = {
                let mut bin_path = None;
                if let Ok(path) = std::env::var("GST_PLUGIN_SCANNER") {
                    if !path.is_empty() {
                        let mut path = std::path::PathBuf::from(path);
                        path.set_file_name("");
                        bin_path = Some(path);
                    }
                } else if let Ok(path) = std::env::var("GST_PLUGIN_SCANNER_1_0") {
                    if !path.is_empty() {
                        let mut path = std::path::PathBuf::from(path);
                        path.set_file_name("");
                        bin_path = Some(path);
                    }
                }
                bin_path
            };
            if let Some(gst_bin_path) = gst_bin_path {
                let path = if let Ok(path_env) = std::env::var("PATH") {
                    format!("{}:{path_env}", gst_bin_path.to_string_lossy())
                } else {
                    gst_bin_path.to_string_lossy().into_owned()
                };
                command.env("PATH", path);
            } else {
                log::warn!("Cannot determine path to gst-inspect");
            }
            command.envs([("GST_DEBUG", "3"), ("GST_REGISTER", &registry_file_name)]);
            command.arg("-a");

            _ = std::fs::remove_file(&registry_file_name);
            mxl_investigator::misc::exec_cmd_and_dump_pipes(command);
            _ = std::fs::remove_file(&registry_file_name);
        }
    });
    crate::localization::init();

    if let Err(error) = run() {
        error!("{error:?}");
        mxl_investigator::proc_dir::write_report_error(&error);
        return std::process::ExitCode::FAILURE;
    }
    if let Err(error) = mxl_investigator::proc_dir::cleanup() {
        error!("{error:?}");
        mxl_investigator::proc_dir::write_report_error(&error);
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
