use super::version1;
use super::version2;
use crate::about;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PreferencesStorage {
    Version1(version1::PreferencesData),
    Version2(version2::PreferencesData),
}

impl From<PreferencesStorage> for super::PreferencesData {
    fn from(cnf: PreferencesStorage) -> Self {
        match cnf {
            PreferencesStorage::Version1(s) => {
                Self::from(PreferencesStorage::Version2(version2::PreferencesData::from(s)))
            }
            PreferencesStorage::Version2(s) => s,
        }
    }
}

impl From<super::PreferencesData> for PreferencesStorage {
    fn from(s: super::PreferencesData) -> Self {
        PreferencesStorage::Version2(s.clone())
    }
}

#[derive(Default)]
pub struct PreferencesManager {
    data: super::PreferencesData,
}

impl Drop for PreferencesManager {
    fn drop(&mut self) {
        self.save().expect("Cannot save preferences");
    }
}

impl PreferencesManager {
    fn default_preferences_path() -> std::path::PathBuf {
        mxl_base::misc::project_dirs().config_dir().to_path_buf()
    }

    fn default_preferences_file() -> Result<std::path::PathBuf> {
        Ok(Self::default_preferences_path().join(const_format::formatcp!("{}.json", about::BINARY_NAME)))
    }

    pub fn init() -> Result<Self> {
        if Self::default_preferences_file()?.is_file() {
            return Self::load();
        }
        Ok(Self::default())
    }

    fn load() -> Result<Self> {
        let file = Self::default_preferences_file()?;
        let str = std::fs::read_to_string(file.clone())
            .with_context(|| format!("Cannot read preferences file '{}'", file.to_string_lossy()))?;
        let cnf = serde_json::from_str::<PreferencesStorage>(str.as_str())
            .with_context(|| format!("Cannot parse preferences file '{}'", file.to_string_lossy()))?;
        Ok(Self { data: cnf.into() })
    }

    pub fn save(&self) -> Result<()> {
        let cnf: PreferencesStorage = self.data.clone().into();
        let str = serde_json::to_string_pretty(&cnf).with_context(|| "Cannot serialize preferences")?;
        let file = Self::default_preferences_file()?;
        std::fs::create_dir_all(Self::default_preferences_path())?;
        std::fs::write(file.clone(), str)
            .with_context(|| format!("Cannot write preferences file '{}'", file.to_string_lossy()))
    }

    pub fn data_mut(&mut self) -> &mut super::PreferencesData {
        &mut self.data
    }

    pub fn data(&self) -> &super::PreferencesData {
        &self.data
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_next_version() {
        let cnf = super::super::PreferencesData::default();
        _ = super::super::version_next::PreferencesData::from(cnf);
    }
}
