use std::sync::mpsc::TryRecvError;

use crate::{app::App, errors::AppError, state::AppState};
use eframe::egui;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let outcome = match &app.state {
        AppState::Loading { loading_rx } => match loading_rx.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(AppError::Unexpected)),
        },
        _ => {
            log::warn!("Entered loading render without being in loading state!");
            return;
        }
    };

    match outcome {
        None => {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 20.0);
                ui.spinner();
                ui.label("Loading devices...");
            });
        }
        Some(Ok(result)) => {
            log::info!("Devices loaded successfully; starting playback.");
            app.state.transition(AppState::Playing {
                video_supervisor: result.video_supervisor,
                audio_supervisor: result.audio_supervisor,
                stop_flag: result.stop_flag,
            });
        }
        Some(Err(e)) => {
            log::error!("LoadingResult failed! {e}");
            app.state.transition(AppState::Error(e.to_string()));
        }
    }
}
