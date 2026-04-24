use core::time::Duration;
use std::collections::HashSet as StdHashSet;
use std::path::PathBuf;

use notify_debouncer_full::new_debouncer;

use crate::args::Source;
use crate::utils::ParentPath as _;

use super::DebouncerInitError;
use super::WATCH_MAP_FOLDER;
use super::event_handler;

pub fn create_all_debouncers<'a, I>(sources: I, fs_timeout: u64) -> Option<impl Sized>
where
    I: IntoIterator<Item = &'a Source>,
{
    let debouncer_results: Vec<Result<_, _>> =
        { create_debouncers(fs_timeout, build_watch_maps(sources)) };

    if report_errors(&debouncer_results) {
        return None;
    }
    Some(debouncer_results)
}

/// Report all errors and return true if any of them failed.
fn report_errors<A>(results: &[Result<A, DebouncerInitError>]) -> bool {
    let mut failed = false;

    // report all errors, but not consume
    #[allow(clippy::explicit_iter_loop, reason = "Don't consume")]
    for result in results.iter() {
        if let Err(error) = result {
            log::error!("{error}");
            failed = true;
        }
    }

    failed
}

/// Create all debouncers.
///
/// Type of debouncers is not really important, we just need to hold them.
fn create_debouncers(
    fs_timeout: u64,
    file_paths: StdHashSet<PathBuf>,
) -> Vec<Result<impl Sized, DebouncerInitError>> {
    let mut debouncers: Vec<Result<_, _>> = vec![];

    let folder_paths = { WATCH_MAP_FOLDER.pin().keys().cloned().collect::<Vec<_>>() };

    debouncers.extend({
        file_paths
            .into_iter()
            .map(move |path| create_debouncer(&path, fs_timeout, event_handler))
    });

    debouncers.extend({
        folder_paths
            .into_iter()
            .map(move |path| create_debouncer(&path, fs_timeout, event_handler))
    });

    debouncers
}

/// Create debouncer.
fn create_debouncer<F>(
    path: &PathBuf,
    fs_timeout: u64,
    event_handler: F,
) -> Result<impl Sized, DebouncerInitError>
where
    F: notify_debouncer_full::DebounceEventHandler,
{
    let mut debouncer =
        match new_debouncer(Duration::from_millis(fs_timeout), None, event_handler) {
            Ok(debouncer) => debouncer,
            Err(error) => {
                return Err(DebouncerInitError::Init(path.clone(), error));
            }
        };

    match debouncer.watch(path, notify::RecursiveMode::NonRecursive) {
        Ok(()) => {}
        Err(error) => {
            return Err(DebouncerInitError::Init(path.clone(), error));
        }
    }

    Ok(debouncer)
}
/// Build watch folder watch maps and return unique known parent folders for files.
fn build_watch_maps<'a, I>(sources: I) -> StdHashSet<PathBuf>
where
    I: IntoIterator<Item = &'a Source>,
{
    let guard_folder = WATCH_MAP_FOLDER.guard();

    let mut known_file_parents = StdHashSet::new();

    for source in sources {
        match source {
            Source::File(location) => {
                known_file_parents.insert(location.source.parent_path().to_path_buf());
            }
            Source::Folder(location) => {
                WATCH_MAP_FOLDER.insert(
                    location.source.clone(),
                    location.destination.clone(),
                    &guard_folder,
                );
            }
        }
    }

    known_file_parents
}
