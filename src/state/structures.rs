use std::ffi::OsString;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    /// Dead-simple inter-process locking mechanism.
    ///
    /// Technically, it may be raw AtomicBool, as state is already wrapped in `papaya::HashMap`.
    /// And probably it's worth a while to make other fields to be inside Arc as well.
    lock: Arc<AtomicBool>,

    /// Source info such as file stem and extension.
    source_info: SourceInfo,

    /// Current file number.
    pub(super) number: u64,

    /// Current last time. `0` if not registered.
    pub(super) last_time: u128,
}

impl State {
    #[inline]
    pub fn new(source_info: SourceInfo, number: u64, last_time: u128) -> Self {
        Self {
            lock: Arc::new(AtomicBool::new(false)),
            source_info,
            number,
            last_time,
        }
    }

    /// Update state inside acquired lock using `update_fn`.
    pub(super) fn update<F>(&mut self, destination: &Path, update_fn: F)
    where
        F: Fn(&Path, &SourceInfo, u64, u128) -> (u64, u128),
    {
        let lock = self.lock.clone();

        // I'm not really sure if we really need to spin here.
        while lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        // Attempt to acquire lock; if it fails, keep spinning.
        {
            // Because CAS is expensive, on failure we simply load the lock status
            // and retry CAS only when we detect the lock has been released
            while lock.load(Ordering::Relaxed) {}
        }

        let (number, last_time) =
            update_fn(destination, &self.source_info, self.number, self.last_time);

        if last_time > self.last_time {
            self.number = number;
            self.last_time = last_time;
        }

        lock.store(false, Ordering::Relaxed);
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
                self.number,
                time_utils::format_time(self.last_time)
            )
        }?;

        Ok(())
    }
}
