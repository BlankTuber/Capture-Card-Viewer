use eframe::egui;

use crate::app::App;

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    if let Some(frame) = app.latest_frame.swap(None) {
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
