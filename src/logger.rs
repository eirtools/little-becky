use log::{LevelFilter, SetLoggerError};

use crate::args::Verbosity;

/// Setup logging with given logging level.
pub fn setup_logging(log_level: LevelFilter) -> Result<(), SetLoggerError> {
    use simplelog::{ColorChoice, Config, TermLogger};

    TermLogger::init(
        log_level,
        Config::default(),
        simplelog::TerminalMode::Stderr,
        ColorChoice::default(),
    )?;

    log::trace!("Setting log level to {log_level}");

    Ok(())
}

impl From<Verbosity> for log::LevelFilter {
    #[inline]
    fn from(val: Verbosity) -> Self {
        match val {
            Verbosity::Off => Self::Off,
            Verbosity::Error => Self::Error,
            Verbosity::Warning => Self::Warn,
            Verbosity::Info => Self::Info,
            Verbosity::Debug => Self::Debug,
            Verbosity::Trace => Self::Trace,
        }
    }
}
