use std::{
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use arc_swap::ArcSwapOption;
use eframe::egui;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
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
    latest_frame: Arc<ArcSwapOption<RgbFrame>>,
    stop: Arc<AtomicBool>,
    repaint_ctx: Arc<OnceLock<egui::Context>>,
) -> Result<JoinHandle<()>, AppError> {
    let handle = thread::spawn(move || {
        let req_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(
            Resolution::new(1920, 1080),
        ));

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
            if stop.load(Ordering::Relaxed) {
                break;
            }
            match camera.frame() {
                Err(e) => {
                    log::warn!("Failed to grab frame: {e}");
                    break;
                }
                Ok(frame) => {
                    let image = match frame.decode_image::<RgbFormat>() {
                        Ok(img) => img,
                        Err(_) => continue,
                    };

                    let w = image.width();
                    let h = image.height();

                    latest_frame.store(Some(Arc::new((w, h, image.into_raw()))));

                    if let Some(ctx) = repaint_ctx.get() {
                        ctx.request_repaint();
                    }
                }
            }
        }
    });

    Ok(handle)
}
