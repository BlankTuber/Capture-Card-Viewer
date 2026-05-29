use crate::{app::App, errors::AppError, state::start_loading};
use eframe::egui;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let screen_size = ui.ctx().content_rect().size();

    // ---------------- Responsive sizing ----------------
    let title_size = (screen_size.y * 0.05).clamp(28.0, 52.0);
    let body_size = (screen_size.y * 0.016).clamp(13.0, 16.0);
    let combo_text_size = (screen_size.y * 0.017).clamp(14.0, 18.0);

    let combo_width = (screen_size.x * 0.24).clamp(280.0, 420.0);
    let combo_height = (screen_size.y * 0.06).clamp(120.0, 160.0);

    let top_spacing = (screen_size.y * 0.12).clamp(75.0, 150.0);
    let section_gap = (screen_size.y * 0.10).clamp(40.0, 90.0);
    let center_gap = (screen_size.x * 0.04).clamp(24.0, 70.0);

    let button_width = (screen_size.x * 0.12).clamp(140.0, 180.0);
    let button_height = (screen_size.y * 0.05).clamp(40.0, 52.0);

    // ---------------- Header ----------------
    ui.vertical_centered(|ui| {
        ui.add_space(top_spacing);

        ui.heading(
            egui::RichText::new("Capture Card Viewer")
                .size(title_size)
                .strong()
                .color(egui::Color32::from_rgb(245, 245, 245)),
        );

        ui.add_space(section_gap);
    });

    // ---------------- Centered menu row ----------------
    let total_width = ui.available_width();
    let block_width = (combo_width * 2.0) + center_gap;
    let left_padding = ((total_width - block_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(left_padding);

        // ---------------- VIDEO INPUT ----------------
        ui.allocate_ui_with_layout(
            egui::vec2(combo_width, combo_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let mut selected_video = app
                    .settings
                    .video_input
                    .clone()
                    .unwrap_or_else(|| "Select Video Input".to_string());

                egui::ComboBox::from_id_salt("video_cb")
                    .width(combo_width)
                    .selected_text(egui::RichText::new(&selected_video).size(combo_text_size))
                    .show_ui(ui, |ui| {
                        for device in &app.available_video_devices {
                            if ui
                                .selectable_value(&mut selected_video, device.clone(), device)
                                .clicked()
                            {
                                app.settings.video_input = Some(device.clone());
                            }
                        }
                    });

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("Often called USB Video or manufacturer name.")
                        .size(body_size)
                        .color(egui::Color32::from_gray(140)),
                );
            },
        );

        ui.add_space(center_gap);

        // ---------------- AUDIO INPUT ----------------
        ui.allocate_ui_with_layout(
            egui::vec2(combo_width, combo_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let mut selected_audio = "Select Audio Input".to_string();

                if let Some(saved_audio_id) = &app.settings.audio_input
                    && let Some((name, _)) = app
                        .available_audio_inputs
                        .iter()
                        .find(|(_, id)| id == saved_audio_id)
                {
                    selected_audio = name.clone();
                }

                egui::ComboBox::from_id_salt("audio_cb")
                    .width(combo_width)
                    .selected_text(egui::RichText::new(&selected_audio).size(combo_text_size))
                    .show_ui(ui, |ui| {
                        for (name, id) in &app.available_audio_inputs {
                            if ui
                                .selectable_value(&mut selected_audio, name.clone(), name)
                                .clicked()
                            {
                                app.settings.audio_input = Some(id.clone());
                            }
                        }
                    });

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("Usually matches the video device name.")
                        .size(body_size)
                        .color(egui::Color32::from_gray(140)),
                );
            },
        );
    });

    // ---------------- Save Button ----------------
    ui.vertical_centered(|ui| {
        ui.add_space(section_gap);

        let save_btn = ui.add_sized(
            [button_width, button_height],
            egui::Button::new(
                egui::RichText::new("Save")
                    .size(body_size)
                    .strong()
                    .color(egui::Color32::from_rgb(230, 230, 230)),
            )
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 153, 112))),
        );

        if save_btn.clicked() {
            if app.settings.video_input.is_none() || app.settings.audio_input.is_none() {
                log::info!("Need to select all inputs!");
                app.runtime_error = Some(AppError::MissingEntries.to_string());
            } else {
                log::info!("Continiuing to loading state");
                let (loading_state, volume_mutex) = start_loading(
                    &app.settings,
                    app.latest_frame.clone(),
                    app.repaint_ctx.clone(),
                );
                if let Err(e) = app.settings.save(&app.data_dir) {
                    log::error!("Failed to save settings: {e}");
                    app.runtime_error = Some(AppError::SettingsSaveFailed.to_string());
                }
                app.state.transition(loading_state);
                app.volume = volume_mutex;
            }
        }
    });
}
