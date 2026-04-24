use std::thread;

use super::init::create_all_debouncers;
use crate::args::Source;

/// Watch sources.
pub fn watch<'a, I>(sources: I, fs_timeout: u64)
where
    I: IntoIterator<Item = &'a Source>,
{
    let debouncer_results = create_all_debouncers(sources, fs_timeout);
    match debouncer_results {
        Some(_) => {}
        None => return,
    }

    #[allow(clippy::infinite_loop, reason = "Waiting for notify events forever")]
    loop {
        // NEVER RETURN
        thread::park();
    }
}
