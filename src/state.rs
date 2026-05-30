use std::{
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    thread::{self, JoinHandle},
};

use arc_swap::ArcSwapOption;
use eframe::egui;
use nokhwa::utils::CameraIndex;

use crate::{
    audio::{AudioStreams, config::AudioConfig},
    errors::AppError,
    settings::Settings,
    video::{RgbFrame, spawn_video_thread},
};

pub struct LoadingResult {
    pub video_thread: JoinHandle<()>,
    pub audio_streams: AudioStreams,
    pub stop_flag: Arc<AtomicBool>,
}

pub enum AppState {
    Initial,
    Loading {
        loading_rx: Receiver<Result<LoadingResult, AppError>>,
    },
    Playing {
        video_thread: JoinHandle<()>,
        #[allow(dead_code)]
        audio_streams: AudioStreams,
        stop_flag: Arc<AtomicBool>,
    },
    Error(String),
}

impl AppState {
    pub fn transition(&mut self, next: AppState) {
        if let AppState::Playing { stop_flag, .. } = &*self {
            stop_flag.store(true, Ordering::Relaxed);
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
            let video_thread =
                spawn_video_thread(camera_index, latest_frame, stop_flag.clone(), repaint_ctx)?;

            let audio_config = AudioConfig {
                input_device: audio_input,
                output_device: audio_output,
                volume: volume_clone,
            };
            let audio_streams = AudioStreams::start_playback(audio_config)?;

            Ok(LoadingResult {
                video_thread,
                audio_streams,
                stop_flag,
            })
        })();

        tx.send(result).ok();
    });

    (AppState::Loading { loading_rx: rx }, volume)
}
