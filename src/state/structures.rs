use std::ffi::OsString;
use std::path::Path;

use crate::time_utils;

use super::StateInitializeError;

#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// File stem part of source file.
    pub prefix: OsString,

    /// Extension part of source file or an empty string
    pub extension: Option<OsString>,
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
            extension: value.extension().map(|a| a.to_owned()),
        })
    }
}

/// Basic file state information.
#[derive(Debug, Clone)]
pub(super) struct State {
    /// Source info such as file stem and extension.
    source_info: SourceInfo,

    /// Current file number.
    pub(super) file_id: u64,

    /// Current last time. `0` if not registered.
    pub(super) last_time: u128,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StateUpdateResult {
    file_id: u64,
    last_time: u128,
    force_update: bool,
}

impl StateUpdateResult {
    pub fn reset(file_id: u64) -> Self {
        Self {
            file_id,
            last_time: 0,
            force_update: true,
        }
    }

    pub fn new(file_id: u64, last_time: u128) -> Self {
        Self {
            file_id,
            last_time,
            force_update: false,
        }
    }
}

impl State {
    #[inline]
    pub fn new(source_info: SourceInfo, number: u64, last_time: u128) -> Self {
        Self {
            source_info,
            file_id: number,
            last_time,
        }
    }

    /// Update state inside acquired lock using `update_fn`.
    pub(super) fn update<F>(&mut self, destination: &Path, update_fn: F)
    where
        F: Fn(&Path, &SourceInfo, u64, u128) -> StateUpdateResult,
    {
        let StateUpdateResult {
            file_id,
            last_time,
            force_update,
        } = update_fn(destination, &self.source_info, self.file_id, self.last_time);

        // initial setup
        let zero_file_id = file_id == 0 && self.file_id == 0 && last_time > self.last_time;
        // after some backups have been made
        let normal_operation =
            file_id > self.file_id && (force_update || last_time > self.last_time);

        if zero_file_id || normal_operation {
            self.file_id = file_id;
            self.last_time = last_time;
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.last_time == 0 {
            write!(f, "Empty state")
        } else {
            write!(
                f,
                "Number: {:x}, Last time: {}",
                self.file_id,
                time_utils::format_time(self.last_time)
            )
        }?;

        Ok(())
    }
}
