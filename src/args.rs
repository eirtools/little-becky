use std::path::{absolute, Path, PathBuf};

use clap::Parser;

/// Parse and verify arguments
pub fn parse_arguments() -> Result<CommandLineArgs, CliError> {
    let args = CommandLineArgs::parse();
    verify(args)
}

/// Command line arguments
#[derive(Debug, Parser)]
pub struct CommandLineArgs {
    #[clap(short = 'o', long = "output", help = "Output folder to backup files")]
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

/// Wrapper to log::LoggerLevel to implement ValueEnum to list options for a user.
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

/// Verify arguments
fn verify(args: CommandLineArgs) -> Result<CommandLineArgs, CliError> {
    let sources = convert_sources(&args.sources)?;
    validate_destination(&args.destination)?;

    Ok(CommandLineArgs {
        log_level: args.log_level.clone(),
        destination: args.destination.clone(),
        sources,
    })
}

/// Read source filenames and convert them to absolute filenames
///
/// Function also ensures, that files has file stem.
fn convert_sources(sources: &Vec<PathBuf>) -> Result<Vec<PathBuf>, CliError> {
    let mut result: Vec<PathBuf> = vec![];
    for source in sources {
        let source_file = match absolute(source) {
            Ok(source_file) => source_file,
            Err(error) => {
                return Err(CliError::SourceNoAbsolute {
                    filename: source.clone(),
                    error,
                });
            }
        };

        if result.contains(&source_file) {
            continue;
        }

        if !source_file.is_file() {
            return Err(CliError::SourceNotAFile(source.clone()));
        }

        if source_file.file_stem().is_none() {
            return Err(CliError::SourceNoFileStem(source.clone()));
        };

        result.push(source_file);
    }

    Ok(result)
}

/// Check if destination is a folder
fn validate_destination(destination: &Path) -> Result<(), CliError> {
    if !destination.is_dir() {
        return Err(CliError::DestinationFolder {
            filename: destination.to_path_buf(),
        });
    }
    Ok(())
}

#[derive(Debug)]
pub enum CliError {
    SourceNoAbsolute {
        filename: PathBuf,
        error: std::io::Error,
    },
    SourceNotAFile(PathBuf),
    SourceNoFileStem(PathBuf),
    DestinationFolder {
        filename: PathBuf,
    },
}

impl std::error::Error for CliError {}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::SourceNoAbsolute { filename, error } => write!(
                f,
                "Unable to resolve absolute path for \"{filename:?}\": {error:#?}"
            ),
            CliError::SourceNotAFile(filename) => {
                write!(f, "Source path \"{filename:?}\" is not a file.")
            }
            CliError::SourceNoFileStem(filename) => {
                write!(f, "Unable to get file name from {filename:?}.")
            }
            CliError::DestinationFolder { filename } => {
                write!(f, "Destination path \"{filename:?}\" is not a folder.")
            }
        }
    }
}
