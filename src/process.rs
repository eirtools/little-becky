use std::thread;
use std::time::Duration;

use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::copy::backup_path;
use crate::time_utils;

/// Watch sources
pub fn fs_watcher<'a, I>(sources: I) -> bool
where
    I: IntoIterator<Item = &'a std::path::PathBuf>,
{
    let holder: Vec<notify::Result<_>> = sources
        .into_iter()
        .map(|source| {
            let mut debouncer =
                match new_debouncer(Duration::from_millis(100), None, event_fn_debounce) {
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
    for event in events {
        event_fn(event.event);
    }
}

/// Actual event handler
fn event_fn(event: notify::Event) {
    log::trace!("watch Event at {}:\n{event:#?}", time_utils::local_now());

    match event.kind {
        notify::EventKind::Any => return,
        notify::EventKind::Access(_) => return,
        notify::EventKind::Create(_) => return, // notify user?
        notify::EventKind::Modify(_) => {}
        notify::EventKind::Remove(_) => return, // notify user?
        notify::EventKind::Other => return,
    };

    for path in event.paths {
        let current_time = std::time::Instant::now();
        backup_path(&path);
        let elapsed = current_time.elapsed().as_nanos();

        log::debug!("Copy time {elapsed} ns for {path:?}");
    }
}
