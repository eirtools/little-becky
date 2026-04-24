use crate::args::{Location, Source};
use crate::state::StateInitializeError;
use crate::time_utils;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf, absolute};

use super::{SourceInfo, State};

/// Initialize state by reading file information for source files from arguments.
///
/// Return known file locations for initial copy.
pub fn initialize_state<'a, I>(
    sources: I,
) -> Result<Vec<Location>, StateInitializeError>
where
    I: IntoIterator<Item = &'a Source>,
{
    let mut file_locations = vec![];
    let guard = super::STATE.guard();

    let mut insert = |path: PathBuf, state: State| {
        super::STATE.insert(path, state, &guard);
    };
    for source in sources {
        match source {
            Source::File(location) => {
                file_locations.push(location.clone());
                register_file_source(
                    &location.source,
                    &location.destination,
                    &mut insert,
                )?;
            }
            Source::Folder(location) => {
                file_locations.extend(register_folder_source(location, &mut insert)?);
            }
        }
    }

    Ok(file_locations)
}

/// Register file source.
fn register_file_source<F>(
    source: &Path,
    destination: &Path,
    insert: &mut F,
) -> Result<(), StateInitializeError>
where
    F: FnMut(PathBuf, State),
{
    let source_info: SourceInfo = SourceInfo::try_from(source)?;
    let state = scan_existing_backups(destination, source, &source_info)?;
    if state.last_time() > 0 {
        log::info!("Initial known state for {source:?}: {state}");
    }

    insert(source.to_path_buf(), state);

    Ok(())
}

/// Try to Register a path met during watching.
pub fn try_register_path<F>(source: &Path, lookup_fn: F) -> bool
where
    F: FnOnce(&Path) -> Option<PathBuf>,
{
    let guard = super::STATE.guard();

    log::trace!("Registering additional path {source:?}");
    if super::STATE.contains_key(source, &guard) {
        return true;
    }

    let Some(destination) = lookup_fn(source) else {
        return false; // ignore unknown path
    };

    #[allow(
        clippy::let_underscore_must_use,
        reason = "ignore insertion if its already done"
    )]
    let mut insert = |path: PathBuf, state: State| {
        _ = super::STATE.try_insert(path, state, &guard);
    };

    match register_file_source(source, &destination, &mut insert) {
        Ok(()) => {
            log::trace!("Registered additional path: {source:?} ");
            true
        }
        Err(error) => {
            log::error!("Error while registering {source:?}: {error}");
            false
        }
    }
}

/// Register all files (non-recursive) under folder source.
fn register_folder_source<F>(
    location: &Location,
    insert: &mut F,
) -> Result<Vec<Location>, StateInitializeError>
where
    F: FnMut(PathBuf, State),
{
    create_dir_all(&location.destination).map_err(|error| {
        StateInitializeError::ReadDestinationFolder {
            folder: location.destination.clone(),
            error,
        }
    })?;

    let read_dir = location.source.read_dir().map_err(|error| {
        StateInitializeError::ReadDestinationFolder {
            folder: location.source.clone(),
            error,
        }
    })?;

    let mut additional = vec![];

    for dir_entry in read_dir {
        let entry =
            dir_entry.map_err(|error| StateInitializeError::ReadDestinationFolder {
                folder: location.source.clone(),
                error,
            })?;
        let source = absolute(entry.path()).map_err(|error| {
            StateInitializeError::ReadDestinationFolder {
                folder: location.source.clone(),
                error,
            }
        })?;

        if !source.is_file() {
            continue;
        }

        register_file_source(&source, &location.destination, insert)?;

        additional.push(Location {
            source,
            destination: location.destination.clone(),
        });
    }

    Ok(additional)
}

/// Scan existitng backups to populate state.
fn scan_existing_backups(
    destination: &Path,
    source: &Path,
    source_info: &SourceInfo,
) -> Result<State, StateInitializeError> {
    let mut number: u64 = 0;
    let mut last_time: u128 = 0;

    let prefix_with_underscore = {
        let mut prefix = source_info.prefix.clone();
        prefix.push("_");
        prefix.to_str().map(ToOwned::to_owned).ok_or_else(|| {
            StateInitializeError::UTF8ConversionError {
                source: source.to_path_buf(),
            }
        })?
    };

    let extension = source_info.extension.as_deref();

    let read_dir = match destination.read_dir() {
        Ok(read_dir) => read_dir,
        Err(error) => {
            return Err(StateInitializeError::ReadDestinationFolder {
                folder: destination.to_path_buf(),
                error,
            });
        }
    };

    for dir_entry in read_dir {
        let entry = match dir_entry {
            Ok(entry) => entry,
            Err(error) => {
                return Err(StateInitializeError::ReadDestinationFolder {
                    folder: destination.to_path_buf(),
                    error,
                });
            }
        };

        let path = entry.path();

        #[allow(clippy::filetype_is_file, reason = "Only regular files are supported")]
        let is_file = match entry.file_type() {
            Ok(file_type) => file_type.is_file(),
            Err(error) => {
                log::warn!("Unable to get file type for file {path:?}: {error}");
                continue;
            }
        };

        let file_ext = path.extension();

        let Some(filename) = path.file_stem().and_then(|stem| stem.to_str()) else {
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
            Err(error) => {
                log::warn!("Unable to get file timestamp: {path:?}: {error}");
                continue;
            }
        };

        // Either number of last time wins :P
        if value_num >= number {
            number = value_num;
        }
        if last_time_fs > last_time {
            last_time = last_time_fs;
        }
    }

    Ok(State::new(
        destination.to_path_buf(),
        source_info.clone(),
        number,
        last_time,
    ))
}
