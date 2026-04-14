mod copy;
mod error;
mod event_handler;
mod init;
mod watcher;

pub use copy::initial_copy;
pub use watcher::watch;

use error::DebouncerInitError;
use event_handler::event_handler;
use state::WATCH_MAP_FOLDER;

/// Global watcher state
mod state {
    use papaya::HashMap;
    use std::path::PathBuf;
    use std::sync::LazyLock;

    // Folder source to destination required to register additional files in runtime.
    pub(super) static WATCH_MAP_FOLDER: LazyLock<HashMap<PathBuf, PathBuf>> =
        LazyLock::new(HashMap::new);
}
