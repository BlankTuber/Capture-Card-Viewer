use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    thread::{self, JoinHandle},
};

use crate::{
    audio::{AudioStreams, config::AudioConfig},
    errors::AppError,
    settings::Settings,
    video::{RgbFrame, find_video_device, spawn_video_thread},
};

pub struct LoadingResult {
    pub video_thread: JoinHandle<()>,
    pub audio_streams: AudioStreams,
}

pub enum AppState {
    Initial,
    Loading {
        loading_rx: Receiver<Result<LoadingResult, AppError>>,
    },
    #[allow(dead_code)]
    Playing {
        video_thread: JoinHandle<()>,
        audio_streams: AudioStreams,
    },
    Error(String),
}

impl AppState {
    pub fn transition(&mut self, next: AppState) {
        *self = next
    }
}

pub fn start_loading(
    settings: &Settings,
    latest_frame: Arc<Mutex<Option<RgbFrame>>>,
) -> (AppState, Arc<Mutex<f32>>) {
    let volume = Arc::new(Mutex::new(settings.volume));
    let volume_clone = Arc::clone(&volume);

    let video_name = settings.video_input.clone();
    let audio_input = settings.audio_input.clone();
    let audio_output = settings.audio_output.clone();

    let (tx, rx) = std::sync::mpsc::channel();

    thread::spawn(move || {
        let result = (|| {
            let video_name = video_name.ok_or(AppError::VideoDeviceNotFound)?;
            let audio_input = audio_input.ok_or(AppError::AudioDeviceNotFound)?;

            let camera_index = find_video_device(&video_name)?;
            let video_thread = spawn_video_thread(camera_index, latest_frame)?;

            let audio_config = AudioConfig {
                input_device: audio_input,
                output_device: audio_output,
                volume: volume_clone,
            };
            let audio_streams = AudioStreams::start_playback(audio_config)?;

            Ok(LoadingResult {
                video_thread,
                audio_streams,
            })
        })();

        tx.send(result).ok();
    });

    (AppState::Loading { loading_rx: rx }, volume)
}
