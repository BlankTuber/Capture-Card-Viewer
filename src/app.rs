use std::{
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use arc_swap::ArcSwapOption;
use eframe::egui;
use nokhwa::utils::CameraIndex;

use crate::{settings::Settings, state::AppState, ui, video::RgbFrame};

pub struct DeviceLists {
    pub video: Vec<(String, CameraIndex)>,
    pub audio_inputs: Vec<(String, String)>,
    pub audio_outputs: Vec<(String, String)>,
    pub audio_output_default: Option<String>,
}

pub struct VideoChannel {
    pub latest_frame: Arc<ArcSwapOption<RgbFrame>>,
    pub repaint_ctx: Arc<OnceLock<egui::Context>>,
}

impl VideoChannel {
    pub fn new() -> Self {
        Self {
            latest_frame: Arc::new(ArcSwapOption::empty()),
            repaint_ctx: Arc::new(OnceLock::new()),
        }
    }
}

pub struct App {
    pub state: AppState,
    pub settings: Settings,
    pub settings_snapshot: Option<Settings>,
    pub devices: DeviceLists,
    pub video: VideoChannel,
    pub volume: Arc<Mutex<f32>>,
    pub data_dir: PathBuf,
    pub current_frame: Option<egui::TextureHandle>,
    pub show_settings: bool,
    pub is_fullscreen: bool,
    pub runtime_error: Option<String>,
}

pub struct AppInit {
    pub settings: Settings,
    pub devices: DeviceLists,
    pub video: VideoChannel,
    pub volume: Arc<Mutex<f32>>,
    pub data_dir: PathBuf,
}

impl App {
    pub fn new(initial_state: AppState, init: AppInit) -> Self {
        Self {
            current_frame: None,
            show_settings: false,
            is_fullscreen: init.settings.fullscreen,
            runtime_error: None,
            state: initial_state,
            settings: init.settings,
            settings_snapshot: None,
            devices: init.devices,
            video: init.video,
            volume: init.volume,
            data_dir: init.data_dir,
        }
    }

    pub fn create_style(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        let r = egui::CornerRadius::same(8);
        visuals.window_corner_radius = egui::CornerRadius::same(12);
        visuals.menu_corner_radius = egui::CornerRadius::same(8);
        visuals.widgets.noninteractive.corner_radius = r;
        visuals.widgets.inactive.corner_radius = r;
        visuals.widgets.hovered.corner_radius = r;
        visuals.widgets.active.corner_radius = r;
        visuals.widgets.open.corner_radius = r;

        visuals.panel_fill = egui::Color32::from_rgb(20, 22, 26);
        visuals.window_fill = egui::Color32::from_rgb(26, 28, 33);
        visuals.faint_bg_color = egui::Color32::from_rgb(30, 32, 38);
        visuals.extreme_bg_color = egui::Color32::from_rgb(14, 15, 18);

        let dark_magenta = egui::Color32::from_rgb(153, 0, 112);
        let dark_cyan = egui::Color32::from_rgb(0, 153, 112);

        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 38, 45);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(42, 45, 52);

        visuals.widgets.hovered.bg_fill = dark_magenta;
        visuals.widgets.active.bg_fill = dark_cyan;
        visuals.widgets.open.bg_fill = egui::Color32::from_rgb(42, 45, 52);

        visuals.selection.bg_fill = dark_cyan;

        visuals.window_shadow = egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(120),
        };
        visuals.popup_shadow = visuals.window_shadow;

        ctx.set_visuals(visuals);

        let mut style = (*ctx.global_style()).clone();
        style.spacing.item_spacing = egui::vec2(16.0, 16.0);
        style.spacing.button_padding = egui::vec2(24.0, 12.0);
        style.spacing.window_margin = egui::Margin::same(32);
        style.spacing.menu_margin = egui::Margin::same(8);
        ctx.set_global_style(style);
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.video.repaint_ctx.get_or_init(|| ctx.clone());
        let current_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));

        ctx.input(|i| {
            // Toggle Fullscreen
            if i.key_pressed(egui::Key::F11) {
                self.is_fullscreen = !self.is_fullscreen;
            }

            // Toggle Settings
            if i.key_pressed(egui::Key::S)
                && !matches!(self.state, AppState::Initial | AppState::Error(_))
            {
                self.show_settings = !self.show_settings;
            }

            // Contextual "Go Back" / Cancel
            if i.key_pressed(egui::Key::Escape) {
                if self.show_settings {
                    if let Some(snapshot) = self.settings_snapshot.take() {
                        self.settings = snapshot.clone();
                        *self.volume.lock().unwrap() = snapshot.volume;
                    }
                    self.show_settings = false;
                } else if self.is_fullscreen {
                    self.is_fullscreen = false;
                }
            }

            // Exit Application: Ctrl+Q
            if i.modifiers.command && i.key_pressed(egui::Key::Q) {
                self.state.transition(AppState::Exiting);
            }
        });

        if self.is_fullscreen != current_fullscreen {
            if self.is_fullscreen {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::Normal,
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::render(self, ui);
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.settings.fullscreen = self.is_fullscreen;
        self.settings.volume = *self.volume.lock().unwrap();

        if let Err(e) = self.settings.save(&self.data_dir) {
            log::error!("Failed to save settings on exit: {e}");
        } else {
            log::info!("Settings saved on exit!");
        }
    }
}
