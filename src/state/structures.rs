use core::fmt::{Display, Formatter, Result as FmtResult};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::time_utils;

use super::StateInitializeError;

#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// File stem prefix part for backup file.
    pub prefix: OsString,

    /// Extension part of source file if any.
    pub extension: Option<OsString>,
}

#[derive(Debug, Clone, Copy)]
pub struct StateUpdate {
    file_id: u64,
    last_time: u128,
    force_update: bool,
}

/// Basic file state information.
#[derive(Debug, Clone)]
pub(super) struct State {
    destination: PathBuf,
    /// Metadata to construct backup filenames.
    source_info: SourceInfo,

    /// Current backup file number.
    file_id: u64,

    /// Current last modification time of the last backed up file. `0` if not yet
    /// backed up.
    last_time: u128,
}

impl TryFrom<&Path> for SourceInfo {
    type Error = StateInitializeError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let Some(prefix) = value.file_stem() else {
            return Err(Self::Error::NoFileStem {
                source: value.to_path_buf(),
            });
        };

        Ok(Self {
            prefix: prefix.to_owned(),
            extension: value.extension().map(ToOwned::to_owned),
        })
    }
}

impl State {
    #[inline]
    pub const fn new(
        destination: PathBuf,
        source_info: SourceInfo,
        number: u64,
        last_time: u128,
    ) -> Self {
        Self {
            destination,
            source_info,
            file_id: number,
            last_time,
        }
    }

    pub(super) const fn last_time(&self) -> u128 {
        self.last_time
    }

    /// Update state inside acquired lock using `update_fn`.
    pub(super) fn update<F>(&mut self, update_fn: F)
    where
        F: Fn(&Path, &SourceInfo, u64, u128) -> StateUpdate,
    {
        let StateUpdate {
            file_id,
            last_time,
            force_update,
        } = update_fn(
            &self.destination,
            &self.source_info,
            self.file_id,
            self.last_time,
        );

        if file_id == 0 && last_time == 0 {
            return;
        }

        // initial setup
        let zero_file_id =
            file_id == 0 && self.file_id == 0 && last_time > self.last_time;
        // after some backups have been made
        let normal_operation =
            file_id > self.file_id && (force_update || last_time > self.last_time);

        if zero_file_id || normal_operation {
            log::trace!(
                "Update {:?} with new state ({file_id:x}, {last_time})",
                self.source_info.prefix
            );
            self.file_id = file_id;
            self.last_time = last_time;
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Self {
            file_id, last_time, ..
        } = self;
        if last_time == &0 {
            write!(f, "Empty state ({file_id:x})")
        } else {
            write!(
                f,
                "Number: {file_id:x}, Last time: {}",
                time_utils::format_time(self.last_time)
            )
        }?;

        Ok(())
    }
}

impl StateUpdate {
    pub const fn has_update(&self) -> bool {
        self.last_time != 0
    }

    /// Normal update.
    pub fn backup(file_id: u64, last_time: u128) -> Self {
        assert_ne!(last_time, 0, "Last modified time must not be 0");
        Self {
            file_id,
            last_time,
            force_update: false,
        }
    }

    /// Construct reset event.
    pub fn reset(file_id: u64) -> Self {
        assert_ne!(file_id, 0, "File id must not be 0");
        Self {
            file_id,
            last_time: 0,
            force_update: true,
        }
    }

    /// Construct a state update to represent silent error.
    ///
    /// Actual error must be logged beforehand.
    pub(crate) const fn silent_error() -> Self {
        Self {
            file_id: 0,
            last_time: 0,
            force_update: false,
        }
    }
}
