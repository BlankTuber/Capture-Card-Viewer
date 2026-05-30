use std::sync::atomic::Ordering;

use eframe::egui;

use crate::{app::App, state::AppState};

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    if let AppState::Playing {
        video_thread,
        stop_flag,
        ..
    } = &app.state
        && video_thread.is_finished()
        && !stop_flag.load(Ordering::Relaxed)
    {
        app.runtime_error = Some(
            "Video stream disconnected. Update device in settings, or restart the app, to reconnect."
                .to_string(),
        );
    }

    if let Some(frame) = app.video.latest_frame.swap(None) {
        let (w, h, data) = &*frame;
        let color_image = egui::ColorImage::from_rgb([*w as usize, *h as usize], data);

        if let Some(texture) = &mut app.current_frame {
            texture.set(color_image, egui::TextureOptions::LINEAR);
        } else {
            app.current_frame = Some(ui.ctx().load_texture(
                "video_frame",
                color_image,
                egui::TextureOptions::LINEAR,
            ));
        }
    }

    if let Some(texture) = &app.current_frame {
        let available = ui.available_size();
        ui.centered_and_justified(|ui| {
            ui.add(
                egui::Image::new(texture)
                    .fit_to_exact_size(available)
                    .maintain_aspect_ratio(true),
            );
        });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 2.0 - 10.0);
            ui.spinner();
            ui.label("Waiting for first frame...");
        });
        ui.ctx().request_repaint();
    }
}
