use crate::targets::TargetKind;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub cwd: Option<PathBuf>,

    #[arg(long)]
    pub skip_data: bool,

    #[arg(long)]
    pub skip_translations: bool,

    #[arg(long, short)]
    pub force: Vec<TargetKind>,
}
