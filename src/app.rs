use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eframe::egui;

use crate::{settings::Settings, state::AppState, ui, video::RgbFrame};

pub struct App {
    pub state: AppState,
    pub show_settings: bool,
    pub is_fullscreen: bool,
    pub runtime_error: Option<String>,
    pub settings: Settings,
    pub available_video_devices: Vec<String>,
    pub available_audio_inputs: Vec<(String, String)>,
    pub available_audio_outputs: Vec<(String, String)>,
    pub volume: Arc<Mutex<f32>>,
    pub data_dir: PathBuf,
    pub current_frame: Option<egui::TextureHandle>,
    pub latest_frame: Arc<Mutex<Option<RgbFrame>>>,
}

pub struct AppInit {
    pub settings: Settings,
    pub video_devices: Vec<String>,
    pub audio_inputs: Vec<(String, String)>,
    pub audio_outputs: Vec<(String, String)>,
    pub volume: Arc<Mutex<f32>>,
    pub data_dir: PathBuf,
    pub latest_frame: Arc<Mutex<Option<RgbFrame>>>,
}

impl App {
    pub fn new(initial_state: AppState, init: AppInit) -> Self {
        Self {
            state: initial_state,
            show_settings: false,
            is_fullscreen: init.settings.fullscreen,
            runtime_error: None,
            settings: init.settings,
            available_video_devices: init.video_devices,
            available_audio_inputs: init.audio_inputs,
            available_audio_outputs: init.audio_outputs,
            volume: init.volume,
            data_dir: init.data_dir,
            current_frame: None,
            latest_frame: init.latest_frame,
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
        let current_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        let current_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        ctx.input(|i| {
            if i.key_pressed(egui::Key::F11) {
                self.is_fullscreen = !self.is_fullscreen;
            }
            if i.key_pressed(egui::Key::S) {
                self.show_settings = !self.show_settings;
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

        if !self.is_fullscreen && !current_maximized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::render(self, ui);
    }
}
