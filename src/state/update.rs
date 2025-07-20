use std::path::Path;

use crate::state::DESTINATION_ATM;

use super::ProcessError;

/// Update state for specific path using `update_fn`.
pub fn update_state<F>(path: &Path, update_fn: F) -> Result<(), ProcessError>
where
    F: Fn(&Path, &super::structures::SourceInfo, u64, u128) -> (u64, u128),
{
    let destination = match DESTINATION_ATM.get() {
        Some(destination) => destination,
        None => return Err(ProcessError::NoDestination),
    };

    let current_state = super::STATE.pin();
    let path = path.to_owned();

    let update_result = current_state.update(path.clone(), |state| {
        let mut state = state.clone();
        let last_time = state.last_time;
        state.update(&destination, &update_fn);

        if state.last_time != last_time {
            log::info!("Current state for {path:?}: {state}");
        }
        state
    });

    match update_result {
        None => Err(ProcessError::NoSuchPath { path }),
        Some(_) => Ok(()),
    }
}
