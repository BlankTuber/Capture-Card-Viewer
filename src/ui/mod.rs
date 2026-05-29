use eframe::egui;

use crate::{app::App, state::AppState};

mod error_view;
mod initial_view;
mod loading_view;
mod playing_view;
mod settings_view;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    if !app.is_fullscreen {
        egui::Panel::top("menu_bar").show_inside(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Settings").clicked() {
                    app.show_settings = true;
                    ui.close();
                }
            });
        });
    }

    match &app.state {
        AppState::Initial => initial_view::render(app, ui),
        AppState::Loading { .. } => loading_view::render(app, ui),
        AppState::Playing { .. } => playing_view::render(app, ui),
        AppState::Error(_) => error_view::render(app, ui),
    }

    if app.show_settings {
        settings_view::render(app, ui);
    }

    if app.runtime_error.is_some() {
        error_view::render(app, ui);
    }
}
