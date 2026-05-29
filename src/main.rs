use std::{
    fs::create_dir_all,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use directories::BaseDirs;
use eframe::egui;

use crate::{
    app::App,
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
    let video_devices = query_video_devices();
    let audio_inputs = query_audio_inputs(&host);
    let audio_outputs = query_audio_outputs(&host);

    let app = match Settings::load(&data_dir) {
        Ok(settings) => {
            let (app_state, volume) = start_loading(&settings);
            App::new(
                app_state,
                settings,
                video_devices,
                audio_inputs,
                audio_outputs,
                volume,
            )
        }
        Err(_) => {
            let settings = Settings::default();
            let volume = Arc::new(Mutex::new(settings.volume));
            App::new(
                AppState::Initial,
                settings,
                video_devices,
                audio_inputs,
                audio_outputs,
                volume,
            )
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([640.0, 360.0])
            .with_resizable(true)
            .with_maximized(true)
            .with_fullscreen(app.is_fullscreen)
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
