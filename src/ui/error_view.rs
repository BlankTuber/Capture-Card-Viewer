use eframe::egui;

use crate::{
    app::App,
    errors::{RuntimeNotice, Severity},
    settings::Settings,
    state::AppState,
};

pub fn render(app: &mut App, ui: &mut egui::Ui) {
    let error_text = match &app.state {
        AppState::Error(err) => err.clone(),
        _ => {
            log::warn!("Entered error render without being in error state!");
            return;
        }
    };

    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() / 2.0 - 20.0);
        ui.label(egui::RichText::new("A fatal error has occured!").strong());
    });
    ui.add_space(20.0);
    ui.vertical_centered(|ui| ui.label(&error_text));

    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "A log file has been saved at:\n{}",
                app.data_dir.join("app.log").display()
            ))
            .size(12.0)
            .color(egui::Color32::from_gray(140)),
        );
    });

    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(
                "If this keeps happening, resetting your settings may help \
                 (this clears your saved device selections, not your log file).",
            )
            .size(13.0)
            .color(egui::Color32::from_gray(160)),
        );
        ui.add_space(10.0);

        if ui.button("Reset Settings").clicked() {
            log::warn!("User reset settings from the fatal error screen.");
            let defaults = Settings::default();

            if let Err(e) = defaults.save(&app.data_dir) {
                log::error!("Failed to save reset settings: {e}");
                app.runtime_error = Some(RuntimeNotice::error(format!(
                    "Could not reset settings: {e}. You may need to delete settings.toml manually."
                )));

                return;
            }

            app.settings = defaults.clone();
            *app.volume.lock().unwrap() = defaults.volume;
            app.settings_snapshot = None;
            app.show_settings = false;
            app.state.transition(AppState::Initial);
        }
    });
}

pub fn render_modal(app: &mut App, ctx: &egui::Context) {
    let Some(notice) = app.runtime_error.clone() else {
        return;
    };

    let (fill, stroke, label, label_color) = match notice.severity {
        Severity::Info => (
            egui::Color32::from_rgb(20, 40, 30),
            egui::Color32::from_rgb(60, 140, 100),
            "Notice",
            egui::Color32::from_rgb(170, 230, 200),
        ),
        Severity::Error => (
            egui::Color32::from_rgb(40, 20, 20),
            egui::Color32::from_rgb(160, 60, 60),
            "Error",
            egui::Color32::from_rgb(255, 180, 180),
        ),
    };

    let screen_rect = ctx.content_rect();

    let width = (screen_rect.width() * 0.28).clamp(260.0, 420.0);
    let padding = 12.0;

    egui::Area::new(egui::Id::new("runtime_error_toast"))
        .fixed_pos(egui::pos2(screen_rect.right() - width - padding, padding))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(fill)
                .stroke(egui::Stroke::new(1.0_f32, stroke))
                .corner_radius(8.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_width(width);

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(label).strong().color(label_color));

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("Click to dismiss")
                                .size(11.0)
                                .color(egui::Color32::from_gray(160)),
                        );
                    });

                    ui.add_space(6.0);

                    ui.label(
                        egui::RichText::new(&notice.message).color(egui::Color32::from_gray(230)),
                    );

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
