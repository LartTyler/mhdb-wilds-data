use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long)]
    pub config: Option<PathBuf>,
    
    #[arg(long)]
    pub cwd: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run(RunArgs),
}

impl Default for Command {
    fn default() -> Self {
        Self::Run(Default::default())
    }
}

#[derive(Debug, Default, Args)]
pub struct RunArgs {
    #[arg(long)]
    pub skip_data: bool,
    
    #[arg(long)]
    pub skip_translations: bool,
}
