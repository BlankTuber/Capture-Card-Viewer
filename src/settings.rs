use serde::{Deserialize, Serialize};
use std::{
    fs::{read_to_string, write},
    path::Path,
};

use crate::errors::AppError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub video_input: Option<String>, // Stored by device name, as nokhwa does not give ID
    pub audio_input: Option<String>, // Stored as device ID, as it's less bound to changes
    pub audio_output: String,
    pub volume: f32,
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            video_input: None,
            audio_input: None,
            audio_output: String::new(),
            volume: 1.0,
            fullscreen: false,
        }
    }
}

impl Settings {
    pub fn load(data_dir: &Path) -> Result<Settings, AppError> {
        let settings_path = data_dir.join("settings.toml");
        let contents = read_to_string(&settings_path).map_err(|error| {
            log::warn!("Failed to read settings: {error}");
            AppError::SettingsNotFound
        })?;

        toml::from_str::<Settings>(&contents).map_err(|e| {
            log::warn!("Settings file is corrupt: {e}. Deleting and resetting.");
            let _ = std::fs::remove_file(&settings_path);
            AppError::SettingsCorrupt
        })
    }

    pub fn save(&self, data_dir: &Path) -> Result<(), AppError> {
        let settings_path = data_dir.join("settings.toml");
        let toml_string = toml::to_string_pretty(self).map_err(|e| {
            log::warn!("Failed to serialize to TOML: {e}");
            AppError::SettingsSaveFailed
        })?;

        write(settings_path, toml_string).map_err(|e| {
            log::warn!("Failed to save settings: {e}");
            AppError::SettingsSaveFailed
        })?;

        log::debug!("Settings saved successfully.");
        Ok(())
    }
}
