use crate::app::App;
use eframe::egui;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let screen_size = ui.ctx().content_rect().size();

    ui.vertical_centered(|ui| {
        ui.add_space(screen_size.y * 0.20);

        let title_size = (screen_size.y * 0.05).clamp(28.0, 48.0);
        ui.heading(
            egui::RichText::new("Capture Card Viewer")
                .size(title_size)
                .strong()
                .color(egui::Color32::from_rgb(245, 245, 245)),
        );

        ui.add_space(screen_size.y * 0.12);
    });

    let total_width = ui.available_width();
    let combo_width = (total_width * 0.35).clamp(280.0, 400.0);
    let center_gap = 60.0;
    let total_menu_block_width = (combo_width * 2.0) + center_gap;
    let left_padding = ((total_width - total_menu_block_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(left_padding);

        ui.allocate_ui_with_layout(
            egui::vec2(combo_width, 150.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let mut selected_video = app
                    .settings
                    .video_input
                    .clone()
                    .unwrap_or_else(|| "Select Video Input".to_string());

                egui::ComboBox::from_id_salt("video_cb")
                    .width(combo_width)
                    .selected_text(egui::RichText::new(&selected_video).size(15.0))
                    .show_ui(ui, |ui| {
                        for device in &app.available_video_devices {
                            if ui.selectable_value(&mut selected_video, device.clone(), device).clicked() {
                                app.settings.video_input = Some(device.clone());
                            }
                        }
                    });

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("Often called something generic, like \"USB Video\",\nor named after the manufacturer, like \"Elgato ... 1.352.0\"")
                        .size(13.0)
                        .color(egui::Color32::from_gray(140)),
                );
            }
        );

        ui.add_space(center_gap);

        ui.allocate_ui_with_layout(
            egui::vec2(combo_width, 150.0),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let mut selected_audio_name = "Select Audio Input".to_string();
                if let Some(saved_audio_id) = &app.settings.audio_input
                    && let Some((name, _)) = app.available_audio_inputs.iter().find(|(_, id)| id == saved_audio_id) {
                        selected_audio_name = name.clone();
                    }

                egui::ComboBox::from_id_salt("audio_cb")
                    .width(combo_width)
                    .selected_text(egui::RichText::new(&selected_audio_name).size(15.0))
                    .show_ui(ui, |ui| {
                        for (name, id) in &app.available_audio_inputs {
                            if ui.selectable_value(&mut selected_audio_name, name.clone(), name).clicked() {
                                app.settings.audio_input = Some(id.clone());
                            }
                        }
                    });

                ui.add_space(16.0);

                ui.label(
                    egui::RichText::new("Often called something similar to the video input")
                        .size(13.0)
                        .color(egui::Color32::from_gray(140)),
                );
            }
        );
    });

    ui.vertical_centered(|ui| {
        ui.add_space(screen_size.y * 0.10);

        let save_btn = ui.add_sized(
            [160.0, 45.0],
            egui::Button::new(
                egui::RichText::new("Save")
                    .size(16.0)
                    .strong()
                    .color(egui::Color32::from_rgb(230, 230, 230)),
            )
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 153, 112))),
        );

        if save_btn.clicked() {
            log::info!("Save button clicked!");
            log::info!("Video selected: {:?}", app.settings.video_input);
            log::info!("Audio selected: {:?}", app.settings.audio_input);
        }
    });
}
