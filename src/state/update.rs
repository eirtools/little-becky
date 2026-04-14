use std::path::Path;

use super::{ProcessError, StateUpdateResult, DESTINATION_ATM};

/// Update state for specific path using `update_fn`.
pub fn update_state<F>(path: &Path, update_fn: F) -> Result<(), ProcessError>
where
    F: Fn(&Path, &super::structures::SourceInfo, u64, u128) -> StateUpdateResult,
{
    let destination = match DESTINATION_ATM.get() {
        Some(destination) => destination,
        None => return Err(ProcessError::NoDestination),
    };

    let current_state = super::STATE.pin();
    let path = path.to_owned();

    let update_result = current_state.update(path.clone(), |state| {
        let current_time = std::time::Instant::now();
        let mut state = state.clone();
        let last_time = state.last_time;
        state.update(&destination, &update_fn);

        if state.last_time != last_time {
            let elapsed = current_time.elapsed().as_nanos();

            log::debug!("Process time {elapsed} ns for {path:?}");
            log::info!("Current state for {path:?}: {state}");
        }
        state
    });

    match update_result {
        None => Err(ProcessError::NoSuchPath { path }),
        Some(_) => Ok(()),
    }
}
