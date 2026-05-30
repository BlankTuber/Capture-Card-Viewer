use crate::{app::App, errors::AppError, state::start_loading};
use eframe::egui;

pub fn render(app: &mut App, ctx: &egui::Context) {
    if !app.show_settings {
        return;
    }

    if app.settings_snapshot.is_none() {
        app.settings_snapshot = Some(app.settings.clone());
    }
    let initial_video_input = app.settings_snapshot.as_ref().unwrap().video_input.clone();
    let initial_audio_input = app.settings_snapshot.as_ref().unwrap().audio_input.clone();
    let initial_audio_output = app.settings_snapshot.as_ref().unwrap().audio_output.clone();

    let screen_size = ctx.content_rect().size();

    // ---------------- Responsive sizing ----------------
    let window_width = (screen_size.x * 0.42).clamp(520.0, 760.0);
    let combo_width = (window_width * 0.52).clamp(240.0, 420.0);

    let title_size = (screen_size.y * 0.028).clamp(18.0, 28.0);
    let label_size = (screen_size.y * 0.018).clamp(14.0, 17.0);
    let body_size = (screen_size.y * 0.016).clamp(13.0, 15.0);

    let spacing_y = (screen_size.y * 0.018).clamp(14.0, 26.0);
    let section_gap = (screen_size.y * 0.025).clamp(18.0, 36.0);

    // ---------------- Window ----------------
    egui::Window::new("")
        .id(egui::Id::new("settings_window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_width(window_width);

            // ---------------- Header ----------------
            ui.vertical_centered(|ui| {
                ui.add_space(screen_size.y * 0.01);

                ui.heading(
                    egui::RichText::new("Settings")
                        .size(title_size)
                        .strong()
                        .color(egui::Color32::from_rgb(245, 245, 245)),
                );

                ui.add_space(6.0);

                ui.label(
                    egui::RichText::new("Configure your capture card and audio settings")
                        .size(body_size)
                        .color(egui::Color32::from_gray(160)),
                );
            });

            ui.add_space(section_gap);

            // ---------------- Settings Grid ----------------
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([(screen_size.x * 0.025).clamp(28.0, 48.0), spacing_y])
                .min_col_width(combo_width)
                .show(ui, |ui| {
                    // -------- VIDEO INPUT --------
                    ui.label(egui::RichText::new("Video Input").size(label_size).strong());

                    let mut selected_video = app
                        .settings
                        .video_input
                        .clone()
                        .unwrap_or_else(|| "None".to_string());

                    egui::ComboBox::from_id_salt("set_video_cb")
                        .width(combo_width)
                        .selected_text(egui::RichText::new(&selected_video).size(body_size))
                        .show_ui(ui, |ui| {
                            for device in &app.devices.video {
                                if ui
                                    .selectable_value(&mut selected_video, device.clone(), device)
                                    .clicked()
                                {
                                    app.settings.video_input = Some(device.clone());
                                }
                            }
                        });

                    ui.end_row();

                    // -------- AUDIO INPUT --------
                    ui.label(egui::RichText::new("Audio Input").size(label_size).strong());

                    let mut selected_audio_in_name = "None".to_string();

                    if let Some(saved_id) = &app.settings.audio_input
                        && let Some((name, _)) = app
                            .devices
                            .audio_inputs
                            .iter()
                            .find(|(_, id)| id == saved_id)
                    {
                        selected_audio_in_name = name.clone();
                    }

                    egui::ComboBox::from_id_salt("set_audio_in_cb")
                        .width(combo_width)
                        .selected_text(egui::RichText::new(&selected_audio_in_name).size(body_size))
                        .show_ui(ui, |ui| {
                            for (name, id) in &app.devices.audio_inputs {
                                if ui
                                    .selectable_value(
                                        &mut selected_audio_in_name,
                                        name.clone(),
                                        name,
                                    )
                                    .clicked()
                                {
                                    app.settings.audio_input = Some(id.clone());
                                }
                            }
                        });

                    ui.end_row();

                    // -------- AUDIO OUTPUT --------
                    ui.label(
                        egui::RichText::new("Audio Output")
                            .size(label_size)
                            .strong(),
                    );

                    let selected_audio_out_name = if app.settings.audio_output.is_empty() {
                        "Use System Default".to_string()
                    } else {
                        app.devices
                            .audio_outputs
                            .iter()
                            .find(|(_, id)| *id == app.settings.audio_output)
                            .map(|(name, _)| name.clone())
                            .unwrap_or_else(|| "Use System Default".to_string())
                    };

                    egui::ComboBox::from_id_salt("set_audio_out_cb")
                        .width(combo_width)
                        .selected_text(
                            egui::RichText::new(&selected_audio_out_name).size(body_size),
                        )
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.settings.audio_output,
                                String::new(),
                                "Use System Default",
                            );

                            for (name, id) in &app.devices.audio_outputs {
                                let mut label = name.clone();

                                if let Some(default_id) = &app.devices.audio_output_default
                                    && default_id == id.as_str()
                                {
                                    label.push_str(" (Default)");
                                }

                                ui.selectable_value(
                                    &mut app.settings.audio_output,
                                    id.clone(),
                                    label,
                                );
                            }
                        });

                    ui.end_row();

                    // -------- VOLUME --------
                    ui.label(egui::RichText::new("Volume").size(label_size).strong());

                    ui.add_sized(
                        [combo_width, 24.0],
                        egui::Slider::new(&mut *app.volume.lock().unwrap(), 0.0..=3.0)
                            .text("x")
                            .show_value(true),
                    );

                    ui.end_row();

                    // -------- KEYBINDS --------
                    ui.label(
                        egui::RichText::new("Fullscreen Keybind")
                            .size(label_size)
                            .strong(),
                    );

                    ui.label(
                        egui::RichText::new("F11")
                            .size(body_size)
                            .color(egui::Color32::from_gray(210)),
                    );

                    ui.end_row();

                    ui.label(
                        egui::RichText::new("Settings Keybind")
                            .size(label_size)
                            .strong(),
                    );

                    ui.label(
                        egui::RichText::new("S")
                            .size(body_size)
                            .color(egui::Color32::from_gray(210)),
                    );

                    ui.end_row();
                });

            ui.add_space(section_gap);

            // ---------------- Buttons ----------------
            ui.horizontal_centered(|ui| {
                let button_size = [
                    (window_width * 0.18).clamp(100.0, 150.0),
                    (screen_size.y * 0.05).clamp(36.0, 50.0),
                ];

                let close_btn = ui.add_sized(
                    button_size,
                    egui::Button::new(egui::RichText::new("Close").size(body_size).strong()),
                );

                ui.add_space(12.0);

                let save_btn = ui.add_sized(
                    button_size,
                    egui::Button::new(
                        egui::RichText::new("Save")
                            .size(body_size)
                            .strong()
                            .color(egui::Color32::from_rgb(230, 230, 230)),
                    )
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 153, 112))),
                );

                if close_btn.clicked() {
                    app.settings.video_input = initial_video_input.clone();
                    app.settings.audio_input = initial_audio_input.clone();
                    app.settings.audio_output = initial_audio_output.clone();
                    *app.volume.lock().unwrap() = app.settings.volume;
                    app.settings_snapshot = None;
                    app.show_settings = false;
                }

                if save_btn.clicked() {
                    app.settings.volume = *app.volume.lock().unwrap();

                    if app.settings.video_input.is_none() || app.settings.audio_input.is_none() {
                        app.settings.video_input = initial_video_input.clone();
                        app.settings.audio_input = initial_audio_input.clone();
                        app.runtime_error = Some(AppError::MissingEntries.to_string());
                        return;
                    }

                    if let Err(e) = app.settings.save(&app.data_dir) {
                        log::error!("Failed to save settings: {e}");
                        app.runtime_error = Some(AppError::SettingsSaveFailed.to_string());
                        return;
                    }

                    if app.settings.video_input != initial_video_input
                        || app.settings.audio_input != initial_audio_input
                        || app.settings.audio_output != initial_audio_output
                    {
                        let (loading_state, volume) = start_loading(
                            &app.settings,
                            app.video.latest_frame.clone(),
                            app.video.repaint_ctx.clone(),
                        );
                        app.video.latest_frame.store(None);
                        app.current_frame = None;
                        app.state.transition(loading_state);
                        app.volume = volume;
                    }
                    app.settings_snapshot = None;
                    app.show_settings = false;
                }
            });

            ui.add_space(screen_size.y * 0.01);
        });
}
