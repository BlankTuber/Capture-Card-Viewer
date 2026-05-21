# Capture-Card-Viewer

A simple media viewing tool for standard capture cards built in Rust.

## Crates

- anyhow
- cpal
- directories
- eframe
- log
- nokhwa (features: "input-native")
- ringbuf
- rubato
- serde (features: "derive")
- simplelog
- toml

## File structure

```md
src/
 ├─ main.rs              ← entry point; init logger, load settings, start eframe
 ├─ app.rs               ← core App struct; implements eframe::App
 ├─ state.rs             ← AppState enum and transitions
 ├─ settings.rs          ← Settings struct; load, save, deserialize
 ├─ errors.rs            ← app-wide error types
 ├─ video.rs             ← video thread; grab frames, decode to RGB, send to GUI
 │
 ├─ ui/
 │   ├─ mod.rs           ← dispatches rendering based on current AppState
 │   ├─ initial_view.rs  ← first-launch settings picker
 │   ├─ loading_view.rs  ← loading screen while searching for devices
 │   ├─ playing_view.rs  ← main view; displays video frames
 │   ├─ settings_view.rs ← settings overlay
 │   └─ error_view.rs    ← error display
 │
 └─ audio/
     ├─ mod.rs           ← declares and re-exports the audio submodules
     ├─ config.rs        ← AudioConfig struct; device names, volume, output selection
     ├─ audio.rs         ← device discovery; find input/output devices from config
     ├─ processing.rs    ← resampling, channel conversion, volume scaling
     └─ playback.rs      ← creates streams, owns ring buffer, runs input→output loop
```

## Functionalities

### main.rs

- Initialize the logger (file + terminal output)
- Query all available devices:
  - Video input devices via nokhwa
  - Audio input devices via cpal
  - Audio output devices via cpal
- Load settings via `Settings::load()`
  - If `Ok` → initial state is `AppState::Loading`
  - If `Err` → initial state is `AppState::Initial`
- Configure and launch eframe with the resolved initial state and queried device lists

### app.rs

- `App` struct
  - `state: AppState`
  - `show_settings: bool`
  - `is_fullscreen: bool`
  - `runtime_error: Option<String>`
  - `settings: Settings`
  - `available_video_devices: Vec<String>`
  - `available_audio_inputs: Vec<String>`
  - `available_audio_outputs: Vec<String>`
  - `volume: Arc<Mutex<f32>>` — shared with audio callback for live volume updates
- `impl App`
  - `new(initial_state: AppState, settings: Settings, video_devices: Vec<String>, audio_inputs: Vec<String>, audio_outputs: Vec<String>) -> Self` — initializes `is_fullscreen` from `settings.fullscreen`, initializes `volume` from `settings.volume`
- `impl eframe::App`
  - `update()` — main GUI loop; handles keybinds, sends `ViewportCommand::Fullscreen` when toggled, delegates rendering to `ui::mod`

### state.rs

- `LoadingResult` struct
  - `video_rx: Receiver<RgbFrame>`
  - `video_thread: JoinHandle`
  - `audio_streams: AudioStreams`
- `AppState` enum
  - `Initial`
  - `Loading { loading_rx: Receiver<Result<LoadingResult, AppError>> }`
  - `Playing { video_rx: Receiver<RgbFrame>, video_thread: JoinHandle, audio_streams: AudioStreams }`
  - `Error(String)`
- `impl AppState`
  - `transition(&mut self, next: AppState)` — handles state changes; dropping the current variant automatically cleans up any handles or channels it owns

### settings.rs

- `Settings` struct
  - `video_input: String`
  - `audio_input: String`
  - `audio_output: String`
  - `volume: f32`
  - `fullscreen: bool`
  - `fullscreen_keybind: String`
  - `settings_keybind: String`
- `impl Default`
  - `audio_output` → system default
  - `volume` → 1.5
  - `fullscreen` → false
  - `fullscreen_keybind` → `"F11"`
  - `settings_keybind` → `"S"`
- `impl Settings`
  - `load() -> Result<Settings>` — locate file via `directories`, read, deserialize with `toml` + `serde`, delete and return `Err(AppError::SettingsCorrupt)` if deserialization fails
  - `save(&self) -> Result<()>` — serialize to `toml` and write to OS config dir

### errors.rs

- `AppError` enum
  - `SettingsCorrupt`
  - `SettingsNotFound`
  - `SettingsSaveFailed`
  - `VideoDeviceNotFound`
  - `AudioDeviceNotFound`
  - `VideoStreamFailed`
  - `AudioStreamFailed`
- `impl Display for AppError` — human readable error messages for the UI
- `impl Error for AppError`

### video.rs

- `RgbFrame` type alias for the decoded frame data passed to the GUI
- `find_video_device(input: &str, devices: &[String]) -> Result<CameraIndex>` — match device name from settings against queried device list
- `query_video_devices() -> Vec<String>` — query all available video input devices via nokhwa
- `spawn_video_thread(device: CameraIndex) -> (JoinHandle, Receiver<RgbFrame>)` — spawns the video thread, returns the handle and receiving end of the frame channel
  - Loop: grab frame → decode to RGB → send via channel
  - Exit cleanly if channel is dropped or frame grab fails
  - Log warnings on failed frames rather than panicking

### ui/mod.rs

- `render(app: &mut App, ctx: &egui::Context)` — matches on `app.state` and delegates to the appropriate view
  - `AppState::Initial` → `initial_view::render()`
  - `AppState::Loading` → `loading_view::render()`
  - `AppState::Playing` → `playing_view::render()`
  - `AppState::Error` → `error_view::render()`
  - If `app.show_settings` is true, renders `settings_view::render()` on top regardless of state
  - If `app.runtime_error` is `Some`, renders `error_view::render()` as an overlay

### ui/initial_view.rs

- `render(app: &mut App, ui: &mut egui::Ui)` — renders the first launch settings picker
  - Dropdown for video input, populated from `app.available_video_devices`
  - Dropdown for audio input, populated from `app.available_audio_inputs`
  - Confirm button — saves settings via `Settings::save()` and transitions to `AppState::Loading`

### ui/loading_view.rs

- `render(app: &mut App, ui: &mut egui::Ui)` — renders the loading screen
  - Display a status message and spinner while the setup thread is running
  - Poll `loading_rx` from `AppState::Loading` each frame for a result from the setup thread
    - If no result yet → keep showing spinner
    - If `Ok(LoadingResult)` → transition to `AppState::Playing { video_rx, video_thread, audio_streams }`
    - If `Err(AppError)` → transition to `AppState::Error` with a relevant message

### ui/playing_view.rs

- `render(app: &mut App, ui: &mut egui::Ui)` — renders the main video feed
  - Receives latest `RgbFrame` from the video channel and displays it
  - If `!app.is_fullscreen` → render a menu bar with a settings button that sets `app.show_settings = true`
  - If `app.runtime_error` is `Some` → render the error overlay via `error_view::render()`
  - If `app.show_settings` → render the settings overlay via `settings_view::render()`

### ui/settings_view.rs

- `render(app: &mut App, ui: &mut egui::Ui)` — renders the settings overlay
  - Dropdown for video input, populated from `app.available_video_devices`
  - Dropdown for audio input, populated from `app.available_audio_inputs`
  - Dropdown for audio output, populated from `app.available_audio_outputs`
  - Volume slider — writes directly to `app.volume` (`Arc<Mutex<f32>>`) for live updates
  - Keybind fields for fullscreen and settings menu — updates `app.settings` directly, takes effect immediately
  - Close button — sets `app.show_settings = false`, saves settings via `Settings::save()`
  - Apply button — if device settings changed, saves settings and transitions to `AppState::Loading` to restart streams

### ui/error_view.rs

- `render(app: &mut App, ui: &mut egui::Ui)` — renders error display
  - If `app.state` is `AppState::Error` → renders a full error screen with the error message, app cannot continue
  - If `app.runtime_error` is `Some` → renders a dismissible error overlay on top of the current view, sets `app.runtime_error = None` on dismiss

### audio/mod.rs

- Declares and re-exports the audio submodules:
  - `pub mod config`
  - `pub mod audio`
  - `pub mod processing`
  - `pub mod playback`
- Re-exports the most commonly used public types for convenience, so the rest of the app can use `audio::AudioStreams` instead of `audio::playback::AudioStreams` etc.

### audio/config.rs

- `AudioConfig` struct
  - `input_device: String`
  - `output_device: String`
  - `volume: Arc<Mutex<f32>>` — shared reference for live volume updates
- `impl AudioConfig`
  - `from_settings(settings: &Settings, volume: Arc<Mutex<f32>>) -> Self` — constructs an `AudioConfig` from the relevant fields of `Settings`, takes the shared volume handle

### audio/audio.rs

- `query_audio_inputs() -> Vec<String>` — query all available audio input devices via cpal
- `query_audio_outputs() -> Vec<String>` — query all available audio output devices via cpal
- `find_audio_input(name: &str, devices: &[String]) -> Result<Device>` — match input device name from settings against queried device list
- `find_audio_output(name: &str, devices: &[String]) -> Result<Device>` — match output device name from settings against queried device list

### audio/processing.rs

- `CHUNK_SIZE: usize` — constant defining the audio chunk size
- `Processor` struct
  - `resampler: Option<Fft<f32>>` — optional resampler, only present if input and output sample rates differ
  - `resample_buf: Vec<f32>` — intermediate buffer for resampled audio
  - `channel_buf: Vec<f32>` — intermediate buffer for channel converted audio
  - `input_channels: usize`
  - `output_channels: usize`
- `impl Processor`
  - `new(input_sample_rate: u32, output_sample_rate: u32, input_channels: u16, output_channels: u16) -> Self` — initializes resampler if sample rates differ, allocates buffers
  - `needs_fixed_chunks(&self) -> bool` — returns true if resampler is active, used to determine input chunk sizing
  - `process_chunk(&mut self, input: &[f32], volume: f32) -> &[f32]` — resamples if needed, converts channels if needed, applies volume scaling, returns processed audio slice

### audio/playback.rs

- `AudioStreams` struct
  - `input_stream: cpal::Stream` — cpal input stream handle, kept alive for duration of `Playing`
  - `output_stream: cpal::Stream` — cpal output stream handle, kept alive for duration of `Playing`
- `start_playback(config: AudioConfig) -> Result<AudioStreams>`
  - Build input and output cpal streams from `AudioConfig`
  - Create a `ringbuf` ring buffer connecting the two streams
  - Input stream callback — reads audio chunks from cpal, processes via `Processor`, writes to ring buffer
  - Output stream callback — reads from ring buffer, writes to cpal output
  - Both callbacks read volume from `config.volume` (`Arc<Mutex<f32>>`) each chunk
  - Start both streams
  - Return `AudioStreams` with both handles
