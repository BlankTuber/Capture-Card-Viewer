#[cfg(windows)]
mod imp {
    unsafe extern "system" {
        fn SetThreadExecutionState(flags: u32) -> u32;
    }

    const ES_CONTINUOUS: u32 = 0x8000_0000;
    const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
    const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;

    pub fn prevent_sleep() {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
        }
        log::debug!("Sleep prevention enabled.");
    }

    pub fn allow_sleep() {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }
        log::debug!("Sleep prevention disabled.");
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use std::process::{Child, Command};
    use std::sync::Mutex;

    static CAFFEINATE: Mutex<Option<Child>> = Mutex::new(None);

    pub fn prevent_sleep() {
        let mut guard = CAFFEINATE.lock().unwrap();
        if guard.is_some() {
            return;
        }

        match Command::new("caffeinate").args(["-d", "-i"]).spawn() {
            Ok(child) => {
                *guard = Some(child);
                log::debug!("Sleep prevention enabled (caffeinate).");
            }
            Err(err) => {
                log::warn!("Failed to start caffeinate, sleep prevention inactive: {err}");
            }
        }
    }

    pub fn allow_sleep() {
        let mut guard = CAFFEINATE.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
            log::debug!("Sleep prevention disabled.");
        }
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use std::process::{Child, Command};
    use std::sync::Mutex;

    static INHIBITOR: Mutex<Option<Child>> = Mutex::new(None);

    pub fn prevent_sleep() {
        let mut guard = INHIBITOR.lock().unwrap();
        if guard.is_some() {
            return;
        }

        match Command::new("systemd-inhibit")
            .args([
                "--what=idle:sleep:handle-lid-switch",
                "--who=Capture Card Viewer",
                "--why=Media playback in progress",
                "--mode=block",
                "sleep",
                "infinity",
            ])
            .spawn()
        {
            Ok(child) => {
                *guard = Some(child);
                log::debug!("Sleep prevention enabled (systemd-inhibit).");
            }
            Err(err) => {
                log::warn!("Failed to start systemd-inhibit, sleep prevention inactive: {err}");
            }
        }
    }

    pub fn allow_sleep() {
        let mut guard = INHIBITOR.lock().unwrap();
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
            log::debug!("Sleep prevention disabled.");
        }
    }
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
mod imp {
    pub fn prevent_sleep() {
        log::warn!("Sleep prevention is not implemented on this platform.");
    }
    pub fn allow_sleep() {}
}

pub use imp::{allow_sleep, prevent_sleep};
