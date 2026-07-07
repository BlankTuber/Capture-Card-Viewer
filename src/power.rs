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

#[cfg(not(windows))]
mod imp {
    pub fn prevent_sleep() {}
    pub fn allow_sleep() {}
}

pub use imp::{allow_sleep, prevent_sleep};
