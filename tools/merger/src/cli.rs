use crate::processor::Processor;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub cwd: Option<PathBuf>,

    #[arg(long, short)]
    pub filter: Vec<Processor>,
}
