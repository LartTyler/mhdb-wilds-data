use clap::Parser;
use crate::cli::Cli;
use crate::config::Config;

mod cli;
mod config;
mod processor;

fn main() -> processor::Result {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref());

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    processor::all(&config)
}
