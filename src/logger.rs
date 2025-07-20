use log::LevelFilter;
use simplelog::TermLogger;

use crate::args::Verbosity;

/// Setup logging with given logging level
pub fn setup_logging(log_level: LevelFilter) {
    TermLogger::init(
        log_level,
        Default::default(),
        simplelog::TerminalMode::Stderr,
        Default::default(),
    )
    .expect("Logger must not be previously set up");

    log::trace!("Setting log level to {log_level}");
}

impl From<Verbosity> for log::LevelFilter {
    fn from(val: Verbosity) -> Self {
        match val {
            Verbosity::Off => log::LevelFilter::Off,
            Verbosity::Error => log::LevelFilter::Error,
            Verbosity::Warning => log::LevelFilter::Warn,
            Verbosity::Info => log::LevelFilter::Info,
            Verbosity::Debug => log::LevelFilter::Debug,
            Verbosity::Trace => log::LevelFilter::Trace,
        }
    }
}
