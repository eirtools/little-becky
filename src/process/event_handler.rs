use std::collections::HashSet as StdHashSet;
use std::path::{Path, PathBuf};

use notify_debouncer_full::DebounceEventResult;

use crate::state::try_register_path;
use crate::utils::ParentPath as _;

use super::WATCH_MAP_FOLDER;
use super::copy::{backup_file, reset_state};

/// Lookup for destination file if file parent is known.
fn destination_lookup(source: &Path) -> Option<PathBuf> {
    WATCH_MAP_FOLDER
        .pin()
        .get(&source.parent_path().to_path_buf())
        .cloned()
}

/// Handler to cover both folders and files.
pub fn event_handler(event: DebounceEventResult) {
    let Some((removed, modified)) = collect_event_files(event) else {
        return;
    };

    // Lookup registration, but not change it.
    removed
        .into_iter()
        .filter(|path| try_register_path(path, |_| None))
        .for_each(|path| {
            reset_state(&path); // sets last_time = 0, keeps number
        });

    // Register a file if parent is registered and try_register is true.
    modified
        .into_iter()
        .filter(|source| try_register_path(source, destination_lookup))
        .for_each(|path| {
            backup_file(&path);
        });
}

/// Collect changed files from a debouncer event, returning removed and modified.
fn collect_event_files(
    debouncer_event: DebounceEventResult,
) -> Option<(StdHashSet<PathBuf>, StdHashSet<PathBuf>)> {
    let events = match debouncer_event {
        Ok(events) => events,
        Err(errors) => {
            for error in errors {
                log::warn!("Watch error: {error:#?}");
            }
            return None;
        }
    };

    let mut modified = StdHashSet::new();
    let mut removed = StdHashSet::new();

    for event in events {
        match event.event.kind {
            notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                modified.extend(
                    event
                        .event
                        .paths
                        .iter()
                        .filter(|&path| path.is_file())
                        .cloned(),
                );
            }
            notify::EventKind::Remove(_) => {
                removed.extend(event.event.paths.iter().cloned());
            }
            notify::EventKind::Any
            | notify::EventKind::Access(_)
            | notify::EventKind::Other => {}
        }
    }

    Some((removed, modified))
}
