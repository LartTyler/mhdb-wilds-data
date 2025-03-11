use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

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
    Msg(MsgArgs),
    User,
}

#[derive(Debug, Args)]
pub struct MsgArgs {
    /// The target directory to scan.
    pub target: PathBuf,

    /// The glob to match against.
    pub glob: String,

    pub pattern: String,

    #[arg(long)]
    pub regex: bool,
}
