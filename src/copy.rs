use ::std::io::ErrorKind;
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
            log::info!("Reset \"{path:?}\" info");
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
            if error.kind() != ErrorKind::NotFound {
                log::error!("Unable to resolve absolute path for \"{path:?}\": {error:#?}");
            }
            return;
        }
    };

    let result = update_state(
        &path,
        |destination, source_info, file_id, last_time| -> StateUpdateResult {
            match try_copy_path(destination, &path, source_info, file_id, last_time) {
                Err(err) => {
                    if err.kind() != ErrorKind::NotFound {
                        log::error!("Unable to copy \"{path:?}\": {err}");
                    }
                    StateUpdateResult::default()
                }
                Ok(result) => result,
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
    let next_number = if last_time > 0 { file_id + 1 } else { file_id };

    let file_last_modified = time_utils::fs_time(source_path)?;

    if file_last_modified <= last_time {
        return Ok(StateUpdateResult::default());
    }

    let mut filename = source_info.prefix.clone();
    filename.push(format!("_{next_number:x}"));

    let mut target_filename = destination.to_path_buf();
    target_filename.push(filename);

    if let Some(extension) = &source_info.extension {
        target_filename.set_extension(extension);
    };

    std::fs::copy(source_path, target_filename)?;

    Ok(StateUpdateResult::new(next_number, file_last_modified))
}
