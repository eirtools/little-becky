use core::error::Error;
use core::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CliError {
    #[cfg(feature = "non-existing-option")]
    SourceNonexistent(PathBuf),
    SourceUnsupported(PathBuf),
    SourceNoAbsolute {
        filename: PathBuf,
        error: IoError,
    },
    SourceNoFileStem(PathBuf),
    DestinationNotFolder(PathBuf),
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SourceNoAbsolute { error, .. } => Some(error),
            #[cfg(feature = "non-existing-option")]
            Self::SourceNonexistent(_) => None,
            Self::SourceUnsupported(_)
            | Self::SourceNoFileStem(_)
            | Self::DestinationNotFolder(_) => None,
        }
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            #[cfg(feature = "non-existing-option")]
            Self::SourceNonexistent(filename) => {
                write!(f, "Source path doesn not exists: \"{filename:?}\"")
            }
            Self::SourceUnsupported(filename) => {
                write!(f, "Source file kind is unsupported: \"{filename:?}\"")
            }
            Self::SourceNoAbsolute { filename, error } => {
                write!(
                    f,
                    "Unable to resolve absolute path for \"{filename:?}\": {error}"
                )
            }
            Self::SourceNoFileStem(filename) => {
                write!(f, "Unable to get file name from {filename:?}.")
            }
            Self::DestinationNotFolder(filename) => {
                write!(f, "Destination path \"{filename:?}\" is not a folder.")
            }
        }
    }
}
