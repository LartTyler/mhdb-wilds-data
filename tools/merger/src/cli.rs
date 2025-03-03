use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub cwd: Option<PathBuf>,
}
