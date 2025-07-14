use crate::about;
use clap::Parser;
use std::path::PathBuf;

const fn about_text() -> &'static str {
    const_format::formatcp!(
        "{} {} {} (released on {})\n\n{}",
        about::ORGANIZATION_NAME,
        about::APP_NAME,
        about::VERSION,
        about::GIT_COMMIT_DATE,
        about::COPYRIGHT
    )
}

#[derive(Parser, Debug)]
#[command(version, about=about_text(), long_about = None)]
pub struct CliArgs {
    /// Export a bug report to the specified zip archive file
    #[arg(long, exclusive(true), value_name("ZIP_FILE"))]
    pub export_report: Option<std::path::PathBuf>,
    /// Update the GStreamer registry
    #[arg(long)]
    pub update_registry: bool,
    /// URI of the media to playback
    #[arg()]
    pub uris: Vec<PathBuf>,
}
