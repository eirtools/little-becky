use crate::state::{StateInitializeError, DESTINATION_ATM};
use crate::time_utils;
use std::collections::HashMap as StdHashMap;
use std::path::{Path, PathBuf};

use super::{SourceInfo, State};

/// Initialize state by reading file information for source files from arguments.
pub fn initialize_state<'a, I>(sources: I, destination: &Path) -> Result<(), StateInitializeError>
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    {
        if let Err(_) = DESTINATION_ATM.set(std::sync::Arc::new(destination.to_path_buf())) {
            return Err(StateInitializeError::DoubleInitialization);
        }
    }

    let result = maximum_known_values(sources, destination)?;

    let current_state = super::STATE.pin();

    current_state.clear();

    for (path, state) in result {
        current_state.insert(path, state);
    }

    Ok(())
}

/// Populate known state for all files.
fn maximum_known_values<'a, I>(
    sources: I,
    destination: &Path,
) -> Result<StdHashMap<PathBuf, State>, StateInitializeError>
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    let mut result: StdHashMap<PathBuf, State> = StdHashMap::new();

    for path in sources {
        let source_info: SourceInfo = path.as_path().try_into()?;
        let state = maximum_known_value(destination, path, &source_info)?;
        log::info!("Initial state for {path:?}: {state}");
        result.insert(path.clone(), state);
    }

    Ok(result)
}

/// Populate known state for one file.
fn maximum_known_value(
    destination: &Path,
    source: &Path,
    source_info: &SourceInfo,
) -> Result<State, StateInitializeError> {
    let mut number: u64 = 0;
    let mut last_time: u128 = 0;
    let prefix_with_underscore = {
        let mut prefix = source_info.prefix.clone();
        prefix.push("_");
        prefix.to_str().map(|a| a.to_owned())
    };
    let Some(prefix_with_underscore) = prefix_with_underscore else {
        log::error!("Unable to convert source prefix to UTF-8 string");
        return Err(StateInitializeError::UTF8ConversionError {
            source: source.to_path_buf(),
        });
    };

    let extension = source_info.extension.as_deref();
    let read_dir = match destination.read_dir() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Unable to read dir {destination:?}: {e}");
            return Err(StateInitializeError::ReadDestination {
                folder: destination.to_path_buf(),
                error: e,
            });
        }
    };

    for file in read_dir {
        let file = match file {
            Ok(v) => v,
            Err(e) => {
                return Err(StateInitializeError::ReadDestination {
                    folder: destination.to_path_buf(),
                    error: e,
                });
            }
        };

        let path = file.path();

        let is_file = match file.file_type() {
            Ok(v) => v.is_file(),
            Err(e) => {
                log::warn!("Unable to get file type for file {path:?}: {e}");
                continue;
            }
        };

        let file_ext = path.extension();

        let Some(filename) = path.file_stem().and_then(|f| f.to_str()) else {
            log::warn!("Unable to get UTF-8 file stem: {path:?}");
            continue;
        };

        if file_ext != extension {
            continue;
        }

        // filename should start with given prefix
        let Some(suffix) = filename.strip_prefix(&prefix_with_underscore) else {
            continue;
        };

        // The rest must be a hex number.
        let Ok(value_num) = u64::from_str_radix(suffix, 16) else {
            continue;
        };

        if !is_file {
            number = value_num + 1;
            log::warn!("Avoiding potential filename collision: {path:?}");
            continue;
        }

        // get last time
        let last_time_fs = match time_utils::fs_time(&path) {
            Ok(last_time_fs) => last_time_fs,
            Err(err) => {
                log::warn!("Unable to get file timestamp: {path:?}: {err}");
                continue;
            }
        };

        // Either number of last time wins :P
        if last_time_fs > last_time || value_num >= number {
            number = value_num;
            last_time = last_time_fs;
        }
    }

    if last_time > 0 {
        Ok(State::new(source_info.clone(), number, last_time))
    } else {
        Ok(State::new(source_info.clone(), 0, 0))
    }
}
