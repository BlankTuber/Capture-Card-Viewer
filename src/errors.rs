use std::fmt::Display;

#[derive(Debug)]
pub enum AppError {
    SettingsCorrupt,
    SettingsNotFound,
    SettingsSaveFailed,
    VideoDeviceNotFound,
    AudioDeviceNotFound,
    // VideoStreamFailed,
    AudioStreamFailed,
    AudioSettingsCorrupt,
    MissingEntries,
    Unexpected,
}

impl std::error::Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let readable_error = match self {
            AppError::SettingsCorrupt => "The settings file is corrupt and will be reset",
            AppError::SettingsNotFound => "The settings file was not found",
            AppError::SettingsSaveFailed => "Failed to save settings",
            AppError::VideoDeviceNotFound => "Could not find video device",
            AppError::AudioDeviceNotFound => "Could not find audio device",
            // AppError::VideoStreamFailed => "Video stream has failed",
            AppError::AudioStreamFailed => "Audio stream has failed",
            AppError::AudioSettingsCorrupt => "Audio settings were not found or are corrupt",
            AppError::MissingEntries => "You need to make sure all fields are selected",
            AppError::Unexpected => "Something unexpected happened!",
        };
        write!(f, "{}", readable_error)
    }
}

pub fn fatal_error(msg: &str) -> ! {
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        unsafe extern "system" {
            fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, utype: u32)
            -> i32;
        }

        let title: Vec<u16> = OsStr::new("Startup Error")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let body: Vec<u16> = OsStr::new(msg)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            MessageBoxW(std::ptr::null_mut(), body.as_ptr(), title.as_ptr(), 0x10); // 0x10 = MB_ICONERROR
        }
    }

    #[cfg(not(windows))]
    eprintln!("Fatal error: {msg}");

    std::process::exit(1);
}
