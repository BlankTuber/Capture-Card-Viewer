use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::{settings::Settings, state::AppState, ui};

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
}

impl App {
    pub fn new(
        initial_state: AppState,
        settings: Settings,
        video_devices: Vec<String>,
        audio_inputs: Vec<(String, String)>,
        audio_outputs: Vec<(String, String)>,
    ) -> Self {
        let is_fullscreen = settings.fullscreen;
        let volume = Arc::new(Mutex::new(settings.volume));
        Self {
            state: initial_state,
            show_settings: false,
            is_fullscreen,
            runtime_error: None,
            settings,
            available_video_devices: video_devices,
            available_audio_inputs: audio_inputs,
            available_audio_outputs: audio_outputs,
            volume,
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F11) {
                self.is_fullscreen = !self.is_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
            }
            if i.key_pressed(egui::Key::S) {
                self.show_settings = !self.show_settings;
            }
        })
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui::render(self, ui);
    }
}
