use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod user_extractor;
pub use user_extractor::*;

mod msg_extractor;
pub use msg_extractor::*;

fn run_command<P: AsRef<Path>, A, S>(path: P, args: A) -> Result<()>
where
    A: IntoIterator<Item = S> + Debug,
    S: AsRef<OsStr>,
{
    let status = Command::new(path.as_ref())
        .args(args)
        .stdout(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed)
    }
}

fn is_output_newer(input: &Path, output: &Path) -> Result<bool> {
    Ok(output.exists() && input.metadata()?.modified()? <= output.metadata()?.modified()?)
}

#[macro_export]
macro_rules! maybe_prefix {
    ($prefix:expr, $path:expr) => {
        if let Some(prefix) = $prefix {
            &prefix.join($path)
        } else {
            $path
        }
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not manipulate file path: {0}")]
    PathManipulation(&'static str),

    #[error("Extract command failed")]
    CommandFailed,

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parsing failed: {0}")]
    Parser(#[from] parser::rsz::Error),

    #[error("Serialization failed: {0}")]
    Serializer(#[from] serde_json::Error),
}

pub trait Extractor: Sync {
    fn extract(&self, in_path: &Path, out_path: &Path, indexes: &[u8]) -> Result<Vec<PathBuf>>;
}
