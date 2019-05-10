use log::LevelFilter;
use pretty_env_logger::formatted_builder;

/// Sets up a logger with the given verbosity and quietness. Panics if a logger is already set up.
pub fn init_logger(verbosity: usize, quiet: bool) {
    let level = if quiet {
        LevelFilter::Off
    } else {
        match verbosity {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };
    formatted_builder().filter_level(level).init()
}
