#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#[cfg(windows)]
#[unsafe(no_mangle)]
pub static NvOptimusEnablement: u32 = 1;

#[cfg(windows)]
#[unsafe(no_mangle)]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use cpal::traits::{DeviceTrait, HostTrait};
use directories::BaseDirs;
use eframe::egui;

use crate::{
    app::{App, AppInit, DeviceLists, VideoChannel},
    audio::io::{query_audio_inputs, query_audio_outputs},
    errors::fatal_error,
    settings::Settings,
    state::{AppState, start_loading},
    video::query_video_devices,
};

mod app;
mod audio;
mod errors;
mod logger;
mod settings;
mod state;
mod ui;
mod video;

fn main() {
    let data_dir: PathBuf = BaseDirs::new()
        .map(|base_dirs| base_dirs.data_local_dir().join(env!("CARGO_PKG_NAME")))
        .unwrap_or_else(|| PathBuf::from("."));

    if let Err(e) = create_dir_all(&data_dir).context("Failed to create data directory") {
        fatal_error(&format!("{e:#}"));
    }

    let log_status = match logger::init(&data_dir) {
        Ok(Some(path)) => format!("Log saved at: {}", path.display()),
        Ok(None) => "Logging to terminal only.".to_string(),
        Err(e) => {
            eprintln!("Warning: {e:#}");
            "Logging failed to initialize.".to_string()
        }
    };

    log::info!(
        "Starting {} v{}.\n- {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        log_status
    );

    let host = cpal::default_host();
    let devices = DeviceLists {
        video: query_video_devices(),
        audio_inputs: query_audio_inputs(&host),
        audio_outputs: query_audio_outputs(&host),
        audio_output_default: host
            .default_output_device()
            .and_then(|d| d.id().ok())
            .map(|id| id.to_string()),
    };

    let video = VideoChannel::new();

    let app = match Settings::load(&data_dir) {
        Ok(settings) => {
            let camera_index = settings
                .video_input
                .as_deref()
                .and_then(|name| video::find_video_device(name, &devices.video).ok());

            let (app_state, volume) = start_loading(
                &settings,
                camera_index,
                video.latest_frame.clone(),
                video.repaint_ctx.clone(),
            );

            let init = AppInit {
                settings,
                devices,
                video,
                volume,
                data_dir,
            };

            App::new(app_state, init)
        }
        Err(_) => {
            let settings = Settings::default();
            let volume = Arc::new(Mutex::new(settings.volume));
            let init = AppInit {
                settings,
                devices,
                video,
                volume,
                data_dir,
            };

            App::new(AppState::Initial, init)
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([640.0, 360.0])
            .with_resizable(true)
            .with_maximized(true)
            .with_decorations(false),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        env!("CARGO_PKG_NAME"),
        native_options,
        Box::new(move |cc| {
            App::create_style(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    ) {
        fatal_error(&format!("Failed to start: {e:#}"));
    }
}
