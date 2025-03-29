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

    #[arg(long, short)]
    pub quiet: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Msg(CommandArgs),
    User(CommandArgs),
}

#[derive(Debug, Args)]
pub struct CommandArgs {
    /// The target directory to scan.
    pub target: PathBuf,

    /// The glob to match against.
    pub glob: String,

    pub pattern: String,

    #[arg(long)]
    pub regex: bool,

    /// Exclude paths that match the pattern
    #[arg(long, short = 'x')]
    pub exclude: Vec<String>,
}
