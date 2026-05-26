use std::str::FromStr;

use cpal::{
    Device, DeviceId, DevicesError, Host,
    traits::{DeviceTrait, HostTrait},
};

use crate::errors::AppError;

fn query_devices(
    devices: Result<impl Iterator<Item = Device>, DevicesError>,
) -> Vec<(String, String)> {
    match devices {
        Err(e) => {
            log::warn!("Failed to enumerate devices: {e}");
            Vec::new()
        }
        Ok(iter) => iter
            .filter_map(|device| {
                let name = device.description().ok()?.name().to_string();
                let id = device.id().ok()?.to_string();
                Some((name, id))
            })
            .collect(),
    }
}

pub fn query_audio_inputs(host: &Host) -> Vec<(String, String)> {
    query_devices(host.input_devices())
}

pub fn query_audio_outputs(host: &Host) -> Vec<(String, String)> {
    query_devices(host.output_devices())
}

pub fn find_audio_input(id_str: &str, host: &Host) -> Result<Device, AppError> {
    if id_str.is_empty() {
        log::warn!("find_audio_input called with empty string");
        return Err(AppError::AudioDeviceNotFound);
    }

    let id = DeviceId::from_str(id_str).map_err(|e| {
        log::warn!("Invalid device ID: {e}");
        AppError::AudioDeviceNotFound
    })?;

    host.device_by_id(&id).ok_or(AppError::AudioDeviceNotFound)
}

pub fn find_audio_output(id_str: &str, host: &Host) -> Result<Device, AppError> {
    if id_str.is_empty() {
        return host
            .default_output_device()
            .ok_or(AppError::AudioDeviceNotFound);
    }

    let id = DeviceId::from_str(id_str).map_err(|e| {
        log::warn!("Invalid device ID: {e}");
        AppError::AudioDeviceNotFound
    })?;

    host.device_by_id(&id).ok_or(AppError::AudioDeviceNotFound)
}
