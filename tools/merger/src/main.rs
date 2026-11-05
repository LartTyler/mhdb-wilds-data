use crate::cli::Cli;
use clap::Parser;
use rslib::config::Config;

mod cli;
mod glob;
mod placeholders;
mod placeholders_old;
mod processor;
mod serde;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref());

    if let Some(cwd) = cli.cwd {
        std::env::set_current_dir(cwd).expect("--cwd option specified an invalid path");
    }

    processor::all(&config, &cli.filter)
}
