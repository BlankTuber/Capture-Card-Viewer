use std::{sync::mpsc::Receiver, thread::JoinHandle};

use crate::{audio::AudioStreams, errors::AppError, video::RgbFrame};

pub struct LoadingResult {
    pub video_rx: Receiver<RgbFrame>,
    pub video_thread: JoinHandle<()>,
    pub audio_streams: AudioStreams,
}

pub enum AppState {
    Initial,
    Loading {
        loading_rx: Receiver<Result<LoadingResult, AppError>>,
    },
    Playing {
        video_rx: Receiver<RgbFrame>,
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
