use std::io::Error;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};

/// Filesystem time in nanoseconds.
pub fn fs_time(file: &Path) -> Result<u128, Error> {
    Ok(as_nanos(file.metadata()?.modified()?))
}

/// Utility function to convert time into nanoseconds.
fn as_nanos(time: SystemTime) -> u128 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |dur| dur.as_nanos())
}

/// Format given time to human-readable string in system local timezone.
pub fn format_time(time: u128) -> String {
    if time > i64::MAX as u128 {
        time.to_string()
    } else {
        let date = DateTime::from_timestamp_nanos(time as i64).with_timezone(&Local);
        date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
}
