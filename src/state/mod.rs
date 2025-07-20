mod error;
mod init;
mod structures;
mod update;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::OnceLock;

pub use self::error::ProcessError;
pub use self::error::StateInitializeError;
pub use self::init::initialize_state;
pub use self::structures::SourceInfo;
pub use self::update::update_state;

use self::structures::State;

use papaya::HashMap as PapayaHashMap;

/// Static state cell.
static STATE: LazyLock<PapayaHashMap<PathBuf, State>> = LazyLock::new(PapayaHashMap::new);

/// Destination folder.
static DESTINATION_ATM: OnceLock<Arc<PathBuf>> = OnceLock::new();
