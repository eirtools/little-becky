mod cli;
mod error;
mod resolve;

pub use cli::Verbosity;
pub use error::CliError;
pub use parser::parse_arguments;
pub use resolve::{Args, Location, Source, verify_resolve};

mod parser {
    use clap::Parser as _;

    use super::cli::CommandLineArgs;
    use super::{Args, CliError, verify_resolve};
    /// Parse and verify arguments.
    pub fn parse_arguments() -> Result<Args, CliError> {
        verify_resolve(CommandLineArgs::parse())
    }
}
