mod args;
mod copy;
mod logger;
mod process;
mod state;
mod time_utils;

fn main() {
    let args = match args::parse_arguments() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    if args.sources.len() == 0 {
        return;
    }

    crate::logger::setup_logging(args.log_level.clone().into());

    if state::initialize_state(&args.sources, &args.destination).is_err() {
        // Unable to read target folder properly
        std::process::exit(2);
    };

    copy::initial_copy(&args.sources);

    if !process::fs_watcher(&args.sources) {
        // Unable to start notification listeners
        std::process::exit(3);
    };
}
