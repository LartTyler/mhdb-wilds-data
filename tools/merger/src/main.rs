use crate::cli::Cli;
use crate::config::Config;
use clap::Parser;

mod cli;
mod config;
mod processor;
mod serde;

fn main() -> processor::Result {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref());

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    processor::all(&config, &cli.filter)
}
