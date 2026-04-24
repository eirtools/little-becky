use std::path::{Path, PathBuf, absolute};

pub use crate::utils::ParentPath as _;

use super::cli::CommandLineArgs;
use super::{CliError, Verbosity};

pub struct Args {
    pub fs_timeout: u64,
    pub log_level: Verbosity,
    pub sources: Vec<Source>,
}

#[derive(Debug, Clone)]
pub enum Source {
    File(Location),
    Folder(Location),
}

#[derive(Debug, Clone)]
pub struct Location {
    pub source: PathBuf,
    pub destination: PathBuf,
}

/// Verify and resolve arguments.
pub fn verify_resolve(args: CommandLineArgs) -> Result<Args, CliError> {
    let sources = convert_sources(
        &args.destination,
        &args.sources,
        #[cfg(feature = "non-existing-option")]
        args.register_nonexistent,
    )?;

    Ok(Args {
        fs_timeout: args.fs_timeout,
        log_level: args.log_level,
        sources,
    })
}

impl Source {
    pub const fn source(&self) -> &PathBuf {
        match self {
            Self::File(location) | Self::Folder(location) => &location.source,
        }
    }
}

fn convert_sources(
    args_destination: &Path,
    sources: &[PathBuf],
    #[cfg(feature = "non-existing-option")] register_nonexistent: bool,
) -> Result<Vec<Source>, CliError> {
    let destination =
        absolute(args_destination).map_err(|error| CliError::SourceNoAbsolute {
            filename: args_destination.to_path_buf(),
            error,
        })?;

    if !destination.is_dir() {
        return Err(CliError::DestinationNotFolder(destination));
    }

    let mut result: Vec<Source> = vec![];

    for source_path in sources {
        let source =
            absolute(source_path).map_err(|error| CliError::SourceNoAbsolute {
                filename: source_path.clone(),
                error,
            })?;

        if result.iter().any(|known| known.source() == &source) {
            log::warn!("Skipping duplicated {source:?}");
            continue;
        }

        let target_base = destination.clone();

        let stem = source
            .file_stem()
            .ok_or_else(|| CliError::SourceNoFileStem(source_path.clone()))?;

        let watch_source = if source.is_dir() {
            Source::Folder(Location {
                source: source.clone(),
                destination: target_base.join(stem),
            })
        } else {
            if !source.exists() {
                #[cfg(feature = "non-existing-option")]
                if register_nonexistent {
                    log::warn!("Path doesn't exist. Assuming it's a file: {source:?}");
                } else {
                    return Err(CliError::SourceNonexistent(source));
                }
                #[cfg(not(feature = "non-existing-option"))]
                {
                    log::warn!("Path doesn't exist. Assuming it's a file: {source:?}");
                }
            } else if !source.is_file() {
                return Err(CliError::SourceUnsupported(source));
            } else { /* create a value */
            }

            Source::File(Location {
                source,
                destination: target_base,
            })
        };

        result.push(watch_source);
    }

    let folder_locations = result
        .iter()
        .filter_map(|known| {
            if let Source::Folder(location) = known {
                Some(location.source.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    result.retain(|source| {
        let Source::File(location) = source else {
            return true;
        };

        let watch_path = location.source.parent_path();

        if folder_locations.iter().any(|other| watch_path == other) {
            log::warn!(
                "Skipping {:?}: already covered by folder source {:?}",
                location.source,
                location.source.parent()
            );
            false
        } else {
            true
        }
    });

    Ok(result)
}
