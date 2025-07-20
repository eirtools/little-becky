use std::path::{absolute, Path, PathBuf};

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
        |destination, source_info, number, last_time| -> (u64, u128) {
            match try_copy_path(destination, &path, source_info, number, last_time) {
                Err(err) => {
                    log::error!("Unable to copy \"{path:?}\": {err}");
                    (0, 0)
                }
                Ok((number, last_time)) => (number, last_time),
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
    number: u64,
    last_time: u128,
) -> std::io::Result<(u64, u128)> {
    let next_number = if last_time > 0 { number + 1 } else { number };

    let file_last_modified = time_utils::fs_time(source_path)?;

    if file_last_modified <= last_time {
        return Ok((0, 0));
    }

    let mut filename = source_info.prefix.clone();
    filename.push(format!("_{next_number:x}"));

    let mut target_filename = destination.to_path_buf();
    target_filename.push(filename);

    if let Some(extension) = &source_info.extension {
        target_filename.set_extension(extension);
    };

    std::fs::copy(source_path, target_filename)?;

    Ok((next_number, file_last_modified))
}
