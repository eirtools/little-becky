mod error;
mod init;
mod structures;
mod update;

use std::path::PathBuf;
use std::sync::LazyLock;

pub use self::error::StateInitializeError;
pub use self::init::{initialize_state, try_register_path};
pub use self::structures::{SourceInfo, StateUpdate};
pub use self::update::update_state;

use self::structures::State;

use papaya::HashMap as PapayaHashMap;

/// Static state cell.
static STATE: LazyLock<PapayaHashMap<PathBuf, State>> =
    LazyLock::new(PapayaHashMap::new);
