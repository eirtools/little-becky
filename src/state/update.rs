use std::path::Path;
use std::time::Instant;

use super::{SourceInfo, StateUpdate};

/// Update state for specific path using `update_fn`.
pub fn update_state<F>(path: &Path, update_fn: F)
where
    F: Fn(&Path, &SourceInfo, u64, u128) -> StateUpdate,
{
    let current_state = super::STATE.pin();

    let update_result = current_state.update(path.to_path_buf(), |state| {
        let current_time = Instant::now();
        #[allow(clippy::shadow_reuse, reason = "Cloned state, original is not needed")]
        let mut state = state.clone();
        let last_time = state.last_time();
        state.update(&update_fn);

        if state.last_time() != last_time {
            let elapsed = current_time.elapsed().as_nanos();

            log::debug!("Process time {elapsed} ns for {path:?}");
            log::info!("Current state for {path:?}: {state}");
        }
        state
    });

    if update_result.is_none() {
        log::error!("Trying to update unregistered path: \"{path:?}\"");
    }
}
