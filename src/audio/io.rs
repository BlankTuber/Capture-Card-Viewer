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
        Ok(iter) => {
            let all: Vec<(String, String)> = iter
                .filter_map(|device| {
                    let name = device.description().ok()?.name().to_string();
                    let id = device.id().ok()?.to_string();
                    Some((name, id))
                })
                .collect();

            dedupe_alsa_aliases(all)
        }
    }
}

#[cfg(target_os = "linux")]
fn dedupe_alsa_aliases(devices: Vec<(String, String)>) -> Vec<(String, String)> {
    fn priority(id: &str) -> u8 {
        let raw = id.strip_prefix("alsa:").unwrap_or(id);
        let prefix = raw.split(':').next().unwrap_or(raw);
        match prefix {
            "pipewire" => 0,
            "default" | "sysdefault" => 1,
            "plughw" => 2,
            "hw" => 3,
            "front" => 4,
            "iec958" => 5,
            p if p.starts_with("surround") => 6,
            _ => 1,
        }
    }

    let mut best: Vec<(String, String)> = Vec::new();
    for (name, id) in devices {
        match best.iter_mut().find(|(n, _)| *n == name) {
            Some(existing) => {
                if priority(&id) < priority(&existing.1) {
                    existing.1 = id;
                }
            }
            None => best.push((name, id)),
        }
    }
    best
}

#[cfg(not(target_os = "linux"))]
fn dedupe_alsa_aliases(devices: Vec<(String, String)>) -> Vec<(String, String)> {
    devices
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
