use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn exec<I, S>(path: &Path, args: I) -> bool where I: IntoIterator<Item = S> + Debug, S: AsRef<OsStr> {
    let status = Command::new(path).args(args).stdout(Stdio::null()).status().unwrap();
    status.success()
}
