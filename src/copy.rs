use std::path::{absolute, Path, PathBuf};

use crate::state::StateUpdateResult;
use crate::state::{update_state, SourceInfo};
use crate::time_utils;

/// Do initial copy for all sources if needed.
pub fn initial_copy<'a, I>(sources: I)
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    for source in sources {
        backup_path(source)
    }
}

pub fn reset_state(path: &PathBuf) {
    let path = match absolute(path) {
        Ok(path) => path,
        Err(error) => {
            log::error!("Unable to resolve absolute path for \"{path:?}\": {error:#?}");
            return;
        }
    };

    let result = update_state(&path, |_, _, file_id, _| {
        StateUpdateResult::reset(file_id + 1)
    });

    match result {
        Ok(_) => {
            log::info!("Reset {path:?}");
        }
        Err(error) => {
            log::error!("{error}");
        }
    }
}

/// Run a backup for a single given path.
pub fn backup_path(path: &PathBuf) {
    let path = match absolute(path) {
        Ok(path) => path,
        Err(error) => {
            log::error!("Unable to resolve absolute path for \"{path:?}\": {error:#?}");
            return;
        }
    };

    let result = update_state(
        &path,
        |destination, source_info, file_id, last_time| -> StateUpdateResult {
            match try_copy_path(destination, &path, source_info, file_id, last_time) {
                Err(err) => {
                    log::error!("Unable to copy \"{path:?}\": {err}");
                    StateUpdateResult::default()
                }
                Ok(result) => {
                    if result.last_time != 0 {
                        log::trace!("Updating {path:?}, got {result:?}");
                    }
                    result
                }
            }
        },
    );

    match result {
        Ok(_) => {}
        Err(error) => {
            log::error!("{error}");
        }
    }
}

/// Try to copy path, return new number and new last modification time.
fn try_copy_path(
    destination: &Path,
    source_path: &PathBuf,
    source_info: &SourceInfo,
    file_id: u64,
    last_time: u128,
) -> std::io::Result<StateUpdateResult> {
    let next_id = if file_id == 0 && last_time == 0 {
        file_id
    } else {
        file_id + 1
    };

    log::trace!("{source_path:?} Next id: {next_id:x}");
    let file_last_modified = time_utils::fs_time(source_path)?;

    // Skip old copy
    if file_last_modified <= last_time {
        log::trace!(
            "no copy: {source_path:?} Next id: {next_id:x}, {file_last_modified}/{last_time}"
        );
        return Ok(StateUpdateResult::default());
    }

    let mut filename = source_info.prefix.clone();
    filename.push(format!("_{next_id:x}"));

    let mut target_filename = destination.to_path_buf();
    target_filename.push(filename);

    if let Some(extension) = &source_info.extension {
        target_filename.set_extension(extension);
    };

    log::trace!("pre-copy: {source_path:?} Next id: {next_id:x}, {file_last_modified}");
    std::fs::copy(source_path, target_filename)?;
    log::trace!("Copied: {source_path:?} Next id: {next_id:x}, {file_last_modified}");
    Ok(StateUpdateResult::new(next_id, file_last_modified))
}
