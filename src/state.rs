use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    thread::{self},
};

use arc_swap::ArcSwapOption;
use eframe::egui;
use nokhwa::utils::CameraIndex;

use crate::{
    audio::{AudioSupervisor, config::AudioConfig},
    errors::AppError,
    settings::Settings,
    video::{RgbFrame, VideoSupervisor},
};

pub struct LoadingResult {
    pub video_supervisor: VideoSupervisor,
    pub audio_supervisor: AudioSupervisor,
    pub stop_flag: Arc<AtomicBool>,
}

pub enum AppState {
    Initial,
    Loading {
        loading_rx: Receiver<Result<LoadingResult, AppError>>,
    },
    Playing {
        video_supervisor: VideoSupervisor,
        audio_supervisor: AudioSupervisor,
        stop_flag: Arc<AtomicBool>,
    },
    Error(String),
    Exiting,
}

impl AppState {
    pub fn transition(&mut self, next: AppState) {
        if let AppState::Playing { stop_flag, .. } = &*self {
            stop_flag.store(true, Ordering::Relaxed);
            crate::power::allow_sleep();
        }
        if matches!(next, AppState::Playing { .. }) {
            crate::power::prevent_sleep();
        }
        *self = next;
    }
}

pub fn start_loading(
    settings: &Settings,
    camera_index: Option<CameraIndex>,
    latest_frame: Arc<ArcSwapOption<RgbFrame>>,
    repaint_ctx: Arc<OnceLock<egui::Context>>,
) -> (AppState, Arc<Mutex<f32>>) {
    let volume = Arc::new(Mutex::new(settings.volume));
    let volume_clone = Arc::clone(&volume);

    let audio_input = settings.audio_input.clone();
    let audio_output = settings.audio_output.clone();

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let result = (|| {
            let camera_index = camera_index.ok_or(AppError::VideoDeviceNotFound)?;
            let audio_input = audio_input.ok_or(AppError::AudioDeviceNotFound)?;

            let stop_flag = Arc::new(AtomicBool::new(false));
            let video_supervisor = VideoSupervisor::start(camera_index, latest_frame, repaint_ctx)?;

            let audio_config = AudioConfig {
                input_device: audio_input,
                output_device: audio_output,
                volume: volume_clone,
            };
            let audio_supervisor = AudioSupervisor::start(audio_config)?;

            Ok(LoadingResult {
                video_supervisor,
                audio_supervisor,
                stop_flag,
            })
        })();

        tx.send(result).ok();
    });

    (AppState::Loading { loading_rx: rx }, volume)
}
