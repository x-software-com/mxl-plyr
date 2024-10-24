#![allow(dead_code)]

pub const QUALIFIER: &str = "com";
pub const ORGANIZATION_NAME: &str = "X-Software";
pub const ORGANIZATION_NAME_FULL: &str = "X-Software GmbH";
pub const ORGANIZATION_DOMAIN: &str = "www.x-software.com";
pub const COPYRIGHT: &str = "Copyright 2017-2024 X-Software GmbH, all rights reserved";

pub const APP_ID: &str = "com.x-software.mxl.plyr";
pub const APP_NAME: &str = "MXL Plyr";

pub const RESOURCES_PATH: &str = "/com/x-software/mxl/plyr";
pub const ICONS_RESOURCES_PATH: &str = const_format::formatcp!("{RESOURCES_PATH}/icons");

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GIT_REVISION_NUMBER: &str = env!("VERGEN_GIT_SHA");
pub const GIT_COMMIT_DATE: &str = env!("VERGEN_GIT_COMMIT_DATE");
pub const GIT_COMMIT_TIMESTAMP: &str = env!("VERGEN_GIT_COMMIT_TIMESTAMP");
