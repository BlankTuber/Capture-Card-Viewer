use eframe::egui;

use crate::{app::App, state::AppState};

mod error_view;
mod initial_view;
mod loading_view;
mod playing_view;
mod settings_view;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    if !app.is_fullscreen {
        egui::Panel::top("menu_bar")
            .frame(egui::Frame::NONE.inner_margin(egui::Margin::symmetric(12, 6)))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    match &app.state {
                        AppState::Initial => {}
                        _ => {
                            let settings_btn = ui.button("Settings");
                            if settings_btn.clicked() {
                                log::info!("Settings klikket!");
                                app.show_settings = !app.show_settings;
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✖").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        ui.add_space(5.0);

                        if ui.button("🗕").clicked() {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });
    }

    match &app.state {
        AppState::Initial => {
            app.show_settings = false;
            initial_view::render(app, ui)
        }
        AppState::Loading { .. } => {
            loading_view::render(app, ui);
        }
        AppState::Playing { .. } => {
            playing_view::render(app, ui);
        }
        AppState::Error(_) => error_view::render(app, ui),
    }

    if app.show_settings {
        settings_view::render(app, ui);
    }

    if app.runtime_error.is_some() {
        error_view::render_modal(app, ui);
    }
}
