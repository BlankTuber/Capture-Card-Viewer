use std::{fs::create_dir_all, path::PathBuf};

use anyhow::Context;
use directories::BaseDirs;

use crate::errors::fatal_error;

mod errors;
mod logger;

fn main() {
    let data_dir: PathBuf = BaseDirs::new()
        .map(|base_dirs| base_dirs.data_local_dir().join(env!("CARGO_PKG_NAME")))
        .unwrap_or_else(|| PathBuf::from("."));

    if let Err(e) = create_dir_all(&data_dir).context("Failed to create data directory") {
        fatal_error(&format!("{e:#}"));
    }

    let log_status = match logger::init(&data_dir) {
        Ok(Some(path)) => format!("Log saved at: {}", path.display()),
        Ok(None) => "Logging to terminal only.".to_string(),
        Err(e) => {
            eprintln!("Warning: {e:#}");
            "Logging failed to initialize.".to_string()
        }
    };

    log::info!(
        "Starting {} v{}.\n- {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        log_status
    );
}
