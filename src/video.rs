use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
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

pub fn query_video_devices() -> Vec<(String, CameraIndex)> {
    match query(ApiBackend::Auto) {
        Ok(cameras) => cameras
            .into_iter()
            .map(|c| (c.human_name(), c.index().clone()))
            .collect(),
        Err(e) => {
            log::warn!("Failed to query video devices: {e}");
            Vec::new()
        }
    }
}

pub fn find_video_device(
    name: &str,
    devices: &[(String, CameraIndex)],
) -> Result<CameraIndex, AppError> {
    if name.is_empty() {
        log::warn!("find_video_device called with empty string");
        return Err(AppError::VideoDeviceNotFound);
    }
    devices
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, idx)| idx.clone())
        .ok_or(AppError::VideoDeviceNotFound)
}

fn spawn_video_stream(
    device: CameraIndex,
    latest_frame: Arc<ArcSwapOption<RgbFrame>>,
    stop: Arc<AtomicBool>,
    repaint_ctx: Arc<OnceLock<egui::Context>>,
) -> Result<JoinHandle<()>, AppError> {
    let (init_tx, init_rx) = std::sync::mpsc::channel::<Result<(), AppError>>();

    let handle = thread::spawn(move || {
        let req_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestResolution(
            Resolution::new(1920, 1080),
        ));

        let mut camera = match Camera::new(device, req_format) {
            Ok(c) => c,
            Err(e) => {
                let _ = init_tx.send(Err(classify_camera_error(&e.to_string())));
                return;
            }
        };

        if let Err(e) = camera.open_stream() {
            let _ = init_tx.send(Err(classify_camera_error(&e.to_string())));
            return;
        }

        if init_tx.send(Ok(())).is_err() {
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

    match init_rx.recv() {
        Ok(Ok(())) => Ok(handle),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::VideoStreamFailed),
    }
}

fn classify_camera_error(msg: &str) -> AppError {
    let lower = msg.to_lowercase();
    if lower.contains("access")
        || lower.contains("denied")
        || lower.contains("0x80070005")
        || lower.contains("in use")
        || lower.contains("busy")
    {
        AppError::VideoDeviceInUse
    } else {
        AppError::VideoStreamFailed
    }
}

pub struct VideoSupervisor {
    current: Arc<Mutex<Option<JoinHandle<()>>>>,
    stream_stop: Arc<AtomicBool>,
    watchdog_stop: Arc<AtomicBool>,
    watchdog: Option<JoinHandle<()>>,
    notice: Arc<Mutex<Option<String>>>,
}

impl VideoSupervisor {
    pub fn start(
        device: CameraIndex,
        latest_frame: Arc<ArcSwapOption<RgbFrame>>,
        repaint_ctx: Arc<OnceLock<egui::Context>>,
    ) -> Result<Self, AppError> {
        let stream_stop = Arc::new(AtomicBool::new(false));

        let handle = spawn_video_stream(
            device.clone(),
            latest_frame.clone(),
            stream_stop.clone(),
            repaint_ctx.clone(),
        )?;

        let current = Arc::new(Mutex::new(Some(handle)));
        let watchdog_stop = Arc::new(AtomicBool::new(false));
        let notice: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let current_clone = current.clone();
        let watchdog_stop_clone = watchdog_stop.clone();
        let notice_clone = notice.clone();
        let stream_stop_clone = stream_stop.clone();

        let watchdog = thread::spawn(move || {
            let mut backoff = Duration::from_secs(1);
            const MAX_BACKOFF: Duration = Duration::from_secs(10);

            loop {
                thread::sleep(Duration::from_millis(500));
                if watchdog_stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                let needs_restart = match &*current_clone.lock().unwrap() {
                    None => true,
                    Some(handle) => handle.is_finished(),
                };

                if !needs_restart {
                    backoff = Duration::from_secs(1);
                    continue;
                }

                log::warn!(
                    "Video stream ended unexpectedly (camera unplugged?); \
                     attempting to reconnect..."
                );

                *current_clone.lock().unwrap() = None;

                match spawn_video_stream(
                    device.clone(),
                    latest_frame.clone(),
                    stream_stop_clone.clone(),
                    repaint_ctx.clone(),
                ) {
                    Ok(new_handle) => {
                        *current_clone.lock().unwrap() = Some(new_handle);
                        log::info!("Video stream reconnected automatically.");
                        *notice_clone.lock().unwrap() =
                            Some("Video reconnected automatically.".to_string());
                        backoff = Duration::from_secs(1);
                    }
                    Err(e) => {
                        log::warn!("Automatic video reconnect failed: {e}. Will retry.");
                        *notice_clone.lock().unwrap() =
                            Some(format!("Video reconnect failed: {e}. Retrying..."));
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                        thread::sleep(backoff);
                    }
                }
            }
        });

        Ok(Self {
            current,
            stream_stop,
            watchdog_stop,
            watchdog: Some(watchdog),
            notice,
        })
    }

    pub fn take_notice(&self) -> Option<String> {
        self.notice.lock().unwrap().take()
    }
}

impl Drop for VideoSupervisor {
    fn drop(&mut self) {
        self.watchdog_stop.store(true, Ordering::Relaxed);
        self.stream_stop.store(true, Ordering::Relaxed);

        *self.current.lock().unwrap() = None;

        self.watchdog.take();
    }
}
