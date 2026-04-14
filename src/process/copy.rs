use std::fs::copy;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf, absolute};

use crate::args::Location;
use crate::state::{SourceInfo, StateUpdate, update_state};
use crate::time_utils;

/// Do initial copy for all sources if needed.
pub fn initial_copy<'a, I>(initial_locations: I)
where
    I: IntoIterator<Item = &'a Location>,
{
    initial_locations
        .into_iter()
        .for_each(|location| backup_file(&location.source));
}

/// Reset known last time for given path on remove events.
pub fn reset_state(event_path: &PathBuf) {
    let absolute = match absolute(event_path) {
        Ok(path) => path,
        Err(error) => {
            log::error!(
                "Unable to resolve absolute path for \"{event_path:?}\": {error:#?}"
            );
            return;
        }
    };

    update_state(&absolute, |_, _, file_id, _| {
        StateUpdate::reset(file_id + 1)
    });
    {
        log::info!("Reset {absolute:?}");
    }
}

/// Run a backup for a single file.
pub fn backup_file(event_path: &PathBuf) {
    let path = match absolute(event_path) {
        Ok(path) => path,
        Err(error) => {
            log::error!(
                "Unable to resolve absolute path for \"{event_path:?}\": {error:#?}"
            );
            return;
        }
    };

    update_state(
        &path,
        |destination, source_info, file_id, last_time| -> StateUpdate {
            match try_copy_path(destination, &path, source_info, file_id, last_time) {
                Err(error) => {
                    log::error!("Unable to copy \"{path:?}\": {error}");
                    StateUpdate::silent_error()
                }
                Ok(result) => {
                    if result.has_update() {
                        log::trace!("Updating {path:?}, got {result:?}");
                    }
                    result
                }
            }
        },
    );
}

/// Try to copy path, return new number and new last modification time.
fn try_copy_path(
    destination: &Path,
    source_path: &PathBuf,
    source_info: &SourceInfo,
    file_id: u64,
    last_time: u128,
) -> IoResult<StateUpdate> {
    let next_id = if file_id == 0 && last_time == 0 {
        file_id
    } else {
        file_id + 1
    };
    log::trace!("{source_path:?} Next id: {next_id:x}");
    let file_last_modified = time_utils::fs_time(source_path)?;

    // Skip old copy
    if file_last_modified <= last_time {
        return Ok(StateUpdate::silent_error());
    }

    let mut filename = source_info.prefix.clone();
    filename.push(format!("_{next_id:x}"));

    let mut target_filename = destination.to_path_buf().join(filename);

    if let Some(extension) = &source_info.extension {
        target_filename.set_extension(extension);
    }

    copy(source_path, target_filename)?;
    Ok(StateUpdate::backup(next_id, file_last_modified))
}
