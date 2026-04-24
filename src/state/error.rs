use core::error::Error;
use core::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;
use std::path::PathBuf;

/// Initialization error.
#[derive(Debug)]
pub enum StateInitializeError {
    /// Source path has no file stem (should not be encountered).
    NoFileStem { source: PathBuf },

    /// Source file name conversion to UTF-8 error.
    ///
    /// It's required for removing prefix.
    UTF8ConversionError { source: PathBuf },

    /// Unable to read destination folder.
    ReadDestinationFolder { folder: PathBuf, error: IoError },
}

impl Error for StateInitializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NoFileStem { .. } | Self::UTF8ConversionError { .. } => None,
            Self::ReadDestinationFolder { error, .. } => Some(error),
        }
    }
}

impl Display for StateInitializeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::NoFileStem { source } => {
                write!(f, "Source filename has no file stem: {source:?}")
            }
            Self::UTF8ConversionError { source } => {
                write!(f, "Unable to convert source filename to UTF-8: {source:?}")
            }
            Self::ReadDestinationFolder { folder, error } => {
                write!(f, "Unable to read dir {folder:?}: {error}")
            }
        }
    }
}
