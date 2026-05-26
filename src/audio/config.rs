use std::sync::{Arc, Mutex};

use cpal::{Device, StreamConfig, traits::DeviceTrait};

use crate::{errors::AppError, settings::Settings};

pub struct AudioConfig {
    pub input_device: String,
    pub output_device: String,
    pub volume: Arc<Mutex<f32>>,
}

impl AudioConfig {
    pub fn from_settings(settings: &Settings, volume: Arc<Mutex<f32>>) -> Result<Self, AppError> {
        Ok(Self {
            input_device: settings
                .audio_input
                .clone()
                .ok_or(AppError::AudioDeviceNotFound)?,
            output_device: settings.audio_output.clone(),
            volume,
        })
    }
}

pub struct NegotiatedConfig {
    pub input_config: StreamConfig,
    pub output_config: StreamConfig,
    pub input_channels: u16,
    pub output_channels: u16,
    pub input_sample_rate: u32,
    pub output_sample_rate: u32,
}

impl NegotiatedConfig {
    pub fn negotiate_configs(
        input_device: &Device,
        output_device: &Device,
    ) -> Result<Self, AppError> {
        let input = input_device.default_input_config().map_err(|e| {
            log::warn!("Failed to get default input config: {e}");
            AppError::AudioSettingsCorrupt
        })?;

        let output = output_device.default_output_config().map_err(|e| {
            log::warn!("Failed to get default output config: {e}");
            AppError::AudioSettingsCorrupt
        })?;

        Ok(Self {
            input_config: input.config(),
            output_config: output.config(),
            input_channels: input.channels(),
            output_channels: output.channels(),
            input_sample_rate: input.sample_rate(),
            output_sample_rate: output.sample_rate(),
        })
    }
}
