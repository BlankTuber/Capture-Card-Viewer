use eframe::egui;

use crate::{app::App, state::AppState};

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let error_text = match &app.state {
        AppState::Error(err) => err,
        _ => {
            log::warn!("Entered error render without being in error state!");
            return;
        }
    };

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 20.0);
        ui.spinner();
        ui.label("A fatal error has occured!");
    });
    ui.add_space(20.0);
    ui.vertical_centered(|ui| ui.label(error_text));
}

pub fn render_modal(app: &mut App, ctx: &egui::Context) {
    let Some(error_text) = app.runtime_error.clone() else {
        return;
    };

    let screen_rect = ctx.content_rect();

    let width = (screen_rect.width() * 0.28).clamp(260.0, 420.0);
    let padding = 12.0;

    egui::Area::new(egui::Id::new("runtime_error_toast"))
        .fixed_pos(egui::pos2(screen_rect.right() - width - padding, padding))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgb(40, 20, 20))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 60, 60)))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_width(width);

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Error")
                                .strong()
                                .color(egui::Color32::from_rgb(255, 180, 180)),
                        );

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("Click to dismiss")
                                .size(11.0)
                                .color(egui::Color32::from_gray(160)),
                        );
                    });

                    ui.add_space(6.0);

                    ui.label(egui::RichText::new(error_text).color(egui::Color32::from_gray(230)));

                    let response = ui.interact(
                        ui.min_rect(),
                        egui::Id::new("runtime_error_toast_click"),
                        egui::Sense::click(),
                    );

                    if response.clicked() {
                        app.runtime_error = None;
                    }
                });
        });
}
