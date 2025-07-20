use std::path::PathBuf;

/// Copy process error
#[derive(Debug)]
pub enum ProcessError {
    /// Given path hasn't been recorded.
    ///
    /// Probably it's a problem with path expansion or file has been moved and system returned new path.
    NoSuchPath { path: PathBuf },

    /// Destination has not been set up.
    NoDestination,
}

/// Initialization error
#[derive(Debug)]
pub enum StateInitializeError {
    /// State has been initialized.
    DoubleInitialization,

    /// Source path has no file stem.
    ///
    /// (should not be encountered)
    NoFileStem { source: PathBuf },

    /// Source file name conversion to UTF-8 error.
    ///
    /// It's required for removing prefix.
    UTF8ConversionError { source: PathBuf },

    /// Unable to read destination folder.
    ReadDestination {
        folder: PathBuf,
        error: std::io::Error,
    },
}

impl std::error::Error for ProcessError {}
impl std::error::Error for StateInitializeError {}

impl std::fmt::Display for StateInitializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateInitializeError::DoubleInitialization => {
                write!(f, "Unexpected double initialization")
            }
            StateInitializeError::NoFileStem { source } => {
                write!(f, "Source filename has no file stem: {source:?}")
            }
            StateInitializeError::UTF8ConversionError { source } => {
                write!(f, "Unable to convert source filename to UTF-8: {source:?}")
            }
            StateInitializeError::ReadDestination { folder, error } => {
                write!(f, "Unable to read dir {folder:?}: {error}")
            }
        }
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::NoSuchPath { path } => {
                write!(f, "Trying to update unregistered path: \"{path:?}\"")
            }
            ProcessError::NoDestination => {
                write!(f, "Destination folder is not configured")
            }
        }
    }
}
