use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode, WriteLogger};

pub fn init(data_dir: &Path) -> Result<Option<PathBuf>> {
    let log_path = data_dir.join("app.log");
    let log_file = File::create(&log_path);

    match log_file {
        Ok(file) => {
            CombinedLogger::init(vec![
                TermLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
                WriteLogger::new(LevelFilter::Info, Config::default(), file),
            ])
            .context("Failed to initialize logger")?;

            Ok(Some(log_path))
        }
        Err(e) => {
            TermLogger::init(
                LevelFilter::Debug,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )
            .context("Failed to initialize logger")?;

            log::warn!("Could not open log file, logging to terminal only: {e:#}");
            Ok(None)
        }
    }
}
