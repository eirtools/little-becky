use std::thread;
use std::time::Duration;

use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::copy::{backup_path, reset_state};

/// Watch sources
pub fn fs_watcher<'a, I>(sources: I, fs_timeout: u64) -> bool
where
    I: IntoIterator<Item = &'a std::path::PathBuf>,
{
    let holder: Vec<notify::Result<_>> = sources
        .into_iter()
        .map(|source| {
            let mut debouncer =
                match new_debouncer(Duration::from_millis(fs_timeout), None, event_fn_debounce) {
                    Ok(debouncer) => debouncer,
                    Err(err) => {
                        log::error!("Debouncer init error for {source:?}: {err:#?}");
                        return Err(err);
                    }
                };
            match debouncer.watch(
                std::path::Path::new(source),
                notify::RecursiveMode::NonRecursive,
            ) {
                Ok(_) => (),
                Err(err) => {
                    log::error!("Debouncer init error for {source:?}: {err:#?}");
                    return Err(err);
                }
            };

            Ok(debouncer)
        })
        .collect();

    if holder.iter().any(|v| v.is_err()) {
        return false;
    }
    loop {
        thread::park();
    }
}

/// Debounce event handler wrapper
fn event_fn_debounce(event: DebounceEventResult) {
    let events = match event {
        Ok(events) => events,
        Err(errors) => {
            for error in errors {
                log::warn!("Watch error: {error:#?}")
            }
            return;
        }
    };

    let mut modified_paths = std::collections::HashSet::new();
    let mut removed_paths = std::collections::HashSet::new();

    for event in events {
        match event.event.kind {
            ::notify::EventKind::Create(_) | ::notify::EventKind::Modify(_) => {
                for path in event.event.paths {
                    modified_paths.insert(path);
                }
            }
            ::notify::EventKind::Remove(_) => {
                for path in event.event.paths {
                    removed_paths.insert(path);
                }
            }
            ::notify::EventKind::Any
            | ::notify::EventKind::Access(_)
            | ::notify::EventKind::Other => continue,
        }
    }

    removed_paths.into_iter().for_each(|path| {
        reset_state(&path); // sets last_time = 0, keeps number
    });

    modified_paths.into_iter().for_each(|path| {
        backup_path(&path); // sets last_time = 0, keeps number
    });
}
