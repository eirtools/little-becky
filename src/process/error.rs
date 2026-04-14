use core::error::Error as StdError;
use core::fmt::{Display, Formatter, Result as FmtResult};
use std::path::PathBuf;

/// Debouncer initialization error.
#[derive(Debug)]
pub(super) enum DebouncerInitError {
    Init(PathBuf, notify::Error),
}

impl StdError for DebouncerInitError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self {
            Self::Init(_, error) => Some(error),
        }
    }
}

impl Display for DebouncerInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Init(path, error) => {
                write!(f, "Debouncer init error for {path:?}: {error:#?}")
            }
        }
    }
}
