use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::Path;
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
        .status()
        .unwrap();

    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed)
    }
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
}
