use std::path::{Path, PathBuf};
use wax::{Glob, GlobError};

type Result<T> = std::result::Result<T, GlobError>;

pub(crate) fn expand<P: AsRef<Path>>(base: P, pattern: &str) -> Result<Vec<PathBuf>> {
    let glob = Glob::new(pattern)?;
    glob.walk_with_behavior(base, 1)
        .map(|v| v.map(|v| v.into_path()).map_err(Into::into))
        .collect::<Result<Vec<_>>>()
}
