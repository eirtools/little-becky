use std::path::PathBuf;

use clap::Parser;

/// Command line arguments.
#[derive(Debug, Parser)]
pub struct CommandLineArgs {
    #[cfg(feature = "non-existing-option")]
    #[clap(long = "force-register", help = "Register nonexistent files")]
    pub register_nonexistent: bool,

    #[clap(
        long = "fs-timeout",
        help = "Timeout for FS notify debouncer (ms)",
        default_value_t = 300
    )]
    pub fs_timeout: u64,

    #[clap(
        short = 'o',
        long = "output",
        help = "Output folder root to backup files"
    )]
    pub destination: PathBuf,

    #[clap(
        long = "log-level",
        help = "Verbosity level",
        default_value_t,
        value_enum
    )]
    pub log_level: Verbosity,

    #[clap(num_args = 1.., help = "Source file(s) to monitor")]
    pub sources: Vec<PathBuf>,
}

/// Wrapper to `log::LoggerLevel` to implement `ValueEnum` to list options for a user.
#[derive(clap::ValueEnum, Clone)] // required for clap::ValueEnum
#[derive(Debug)] // for clap parser
#[derive(Default)] // for set default in easier way
pub enum Verbosity {
    Off,
    Error,
    Warning,
    #[default]
    Info,
    Debug,
    Trace,
}
