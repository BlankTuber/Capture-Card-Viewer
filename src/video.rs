use std::{
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType},
};

use crate::errors::AppError;

pub type RgbFrame = (u32, u32, Vec<u8>);

pub fn query_video_devices() -> Vec<String> {
    match query(ApiBackend::Auto) {
        Ok(cameras) => cameras
            .into_iter()
            .map(|camera| camera.human_name())
            .collect(),
        Err(e) => {
            log::warn!("Failed to query video devices: {e}");
            Vec::new()
        }
    }
}

pub fn find_video_device(name: &str) -> Result<CameraIndex, AppError> {
    if name.is_empty() {
        log::warn!("find_video_device called with empty string");
        return Err(AppError::VideoDeviceNotFound);
    }

    query(ApiBackend::Auto)
        .map_err(|e| {
            log::warn!("Failed to query video devices: {e}");
            AppError::VideoDeviceNotFound
        })?
        .into_iter()
        .find(|c| c.human_name() == name)
        .map(|c| c.index().clone())
        .ok_or(AppError::VideoDeviceNotFound)
}

pub fn spawn_video_thread(
    device: CameraIndex,
) -> Result<(JoinHandle<()>, Receiver<RgbFrame>), AppError> {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = thread::spawn(move || {
        let req_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);

        let mut camera = match Camera::new(device, req_format) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to create camera: {e}");
                return;
            }
        };

        if let Err(e) = camera.open_stream() {
            log::warn!("Failed to open video stream: {e}");
            return;
        }

        loop {
            match camera.frame() {
                Err(e) => {
                    log::warn!("Failed to grab frame: {e}");
                    break;
                }
                Ok(frame) => match frame.decode_image::<RgbFormat>() {
                    Err(e) => log::warn!("Failed to decode frame: {e}"),
                    Ok(image) => {
                        let w = image.width();
                        let h = image.height();
                        if tx.send((w, h, image.into_raw())).is_err() {
                            break;
                        }
                    }
                },
            }
        }
    });

    Ok((handle, rx))
}
