use std::{env, path::PathBuf};

use error_stack::{IntoReport, Result, ResultExt};

use crate::APPLICATION_NAME;

lazy_static! {
    pub static ref LOG_DIR: PathBuf = {
        // Check for the environment variable declaring the log directory
        let log_dir = env::var("SS_LOG_DIR").map(|v| PathBuf::from(v.as_str()));

        // If the log directory environment variable is not found use a default
        // location

        // On linux check for the $XDG_STATE_HOME environment variable first,
        // if not found then coose the default location for $XDG_STATE_HOME
        // at /home/<user>/.local/state
        #[cfg(target_family = "unix")]
        let log_dir = log_dir
            .or(env::var("XDG_STATE_HOME")
                .map(|v| PathBuf::from(v.as_str()))
                .map(|v| v.join(APPLICATION_NAME).join("logs")))
            .or(env::var("HOME")
                .map(|v| PathBuf::from(v.as_str()))
                .map(|v| {
                    v.join(".local")
                        .join("state")
                        .join(APPLICATION_NAME)
                        .join("logs")
                }))
            .expect("Failed to load log directory, no $HOME set");

        // On Windows check for the %appdata% environment variable
        #[cfg(target_family = "windows")]
        let log_dir = log_dir
            .or(env::var("appdata")
                .map(|v| PathBuf::from(v.as_str()))
                .map(|v| v.join(APPLICATION_NAME).join("logs")))
            .expect("Failed to load log directory, no %AppData% set");

        log_dir
    };
}

pub fn setup_log() -> Result<(), log::SetLoggerError> {
    use fern::colors::{Color, ColoredLevelConfig};

    let colour_line = ColoredLevelConfig::new()
        .error(Color::White)
        .warn(Color::White)
        .info(Color::White)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    let colour = colour_line
        .clone()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Blue);

    fern::Dispatch::new()
        .chain(
            fern::Dispatch::new()
                .format(move |out, message, record| {
                    #[cfg(debug_assertions)]
                    out.finish(format_args!(
                        "{colour_line}time={time} target={target} file={file} line={line} level={level} msg={msg:?}\x1B[0m",
                        colour_line = format_args!(
                            "\x1B[{}m",
                            colour_line.get_color(&record.level()).to_fg_str(),
                        ),
                        time = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.9f"),
                        target = record.target(),
                        file = record.file().unwrap_or(""),
                        line = record.line().unwrap_or(0),
                        level = colour.color(record.level()),
                        msg = format!("{}", message).as_str(),
                    ));
                    #[cfg(not(debug_assertions))]
                    out.finish(format_args!(
                        "{colour_line}time={time} target={target} level={level} msg={msg:?}\x1B[0m",
                        colour_line = format_args!(
                            "\x1B[{}m",
                            colour_line.get_color(&record.level()).to_fg_str(),
                        ),
                        time = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.9f"),
                        target = record.target(),
                        level = colour.color(record.level()),
                        msg = format!("{}", message).as_str(),
                    ));
                })
                .chain(std::io::stdout()),
        )
        .chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    #[cfg(debug_assertions)]
                    out.finish(format_args!(
                        "time={time} target={target} file={file} line={line} level={level} msg={msg:?}",
                        time = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.9f"),
                        target = record.target(),
                        file = record.file().unwrap_or(""),
                        line = record.line().unwrap_or(0),
                        level = record.level(),
                        msg = format!("{}", message).as_str(),
                    ));
                    #[cfg(not(debug_assertions))]
                    out.finish(format_args!(
                        "time={time} target={target} level={level} msg={msg:?}",
                        time = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.9f"),
                        target = record.target(),
                        level = record.level(),
                        msg = format!("{}", message).as_str(),
                    ));
                })
                .chain(fern::DateBased::new(
                    LOG_DIR.to_path_buf(),
                    format!("%Y-%m-%d.{}.log", APPLICATION_NAME),
                )),
        )
        .apply()
        .report()
        .attach_printable("Unable to setup logger as a global logger has already been set")
}
