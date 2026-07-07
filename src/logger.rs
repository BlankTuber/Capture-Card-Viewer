use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::LevelFilter;
#[cfg(debug_assertions)]
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use simplelog::{Config, WriteLogger};

use crate::errors::fatal_error;

const MAX_ROTATED_LOGS: u32 = 3;

#[cfg(debug_assertions)]
const FILE_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const FILE_LEVEL: LevelFilter = LevelFilter::Info;

fn rotate_logs(data_dir: &Path) {
    let path_for = |n: u32| {
        if n == 0 {
            data_dir.join("app.log")
        } else {
            data_dir.join(format!("app.log.{n}"))
        }
    };

    let _ = std::fs::remove_file(path_for(MAX_ROTATED_LOGS));
    for n in (0..MAX_ROTATED_LOGS).rev() {
        let from = path_for(n);
        if from.exists() {
            let _ = std::fs::rename(&from, path_for(n + 1));
        }
    }
}


fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown location".to_string());

        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "no panic message available".to_string());

        let backtrace = std::backtrace::Backtrace::force_capture();

        log::error!("PANIC at {location}: {message}\nBacktrace:\n{backtrace}");

        std::thread::sleep(std::time::Duration::from_millis(50));

        fatal_error(&format!(
            "The application crashed unexpectedly and needs to close.\n\n{message}\n\nA log file has been saved and may help diagnose the issue."
        ));
    }));
}

pub fn init(data_dir: &Path) -> Result<Option<PathBuf>> {
    rotate_logs(data_dir);

    let log_path = data_dir.join("app.log");
    let log_file = File::create(&log_path);

    let result = match log_file {
        Ok(file) => {
            #[cfg(debug_assertions)]
            {
                CombinedLogger::init(vec![
                    TermLogger::new(
                        LevelFilter::Debug,
                        Config::default(),
                        TerminalMode::Mixed,
                        ColorChoice::Auto,
                    ),
                    WriteLogger::new(FILE_LEVEL, Config::default(), file),
                ])
                .context("Failed to initialize logger")?;
            }

            #[cfg(not(debug_assertions))]
            {
                WriteLogger::init(FILE_LEVEL, Config::default(), file)
                    .context("Failed to initialize logger")?;
            }

            Some(log_path)
        }
        Err(e) => {
            #[cfg(debug_assertions)]
            {
                TermLogger::init(
                    LevelFilter::Debug,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                )
                .context("Failed to initialize logger")?;
            }


            eprintln!("Warning: could not open log file: {e:#}");
            None
        }
    };

    install_panic_hook();
    Ok(result)
}
