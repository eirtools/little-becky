#![allow(
    clippy::unnecessary_debug_formatting,
    clippy::use_debug,
    reason = "Print escaped paths"
)]

use std::process::exit;

use crate::logger::setup_logging;

mod args;
mod logger;
mod process;
mod state;
mod time_utils;
pub(crate) mod utils;

fn main() {
    let args = match args::parse_arguments() {
        Ok(args) => args,
        Err(error) => {
            #[allow(clippy::print_stderr, reason = "No logger set up yet")]
            {
                eprintln!("Error: {error}");
            };
            exit(1);
        }
    };

    if args.sources.is_empty() {
        return;
    }

    match setup_logging(args.log_level.clone().into()) {
        Ok(()) => {}
        #[allow(clippy::print_stderr, reason = "No logger set up yet")]
        Err(error) => {
            eprintln!("Logger must not be previously set up: {error}");
            exit(1)
        }
    }

    let initial_locations = match state::initialize_state(&args.sources) {
        Ok(additional) => additional,
        Err(error) => {
            log::error!("Initialization error: {error}");
            exit(2);
        }
    };

    process::initial_copy(&initial_locations);
    process::watch(&args.sources, args.fs_timeout);

    // Unable to start notification listeners
    exit(3);
}
