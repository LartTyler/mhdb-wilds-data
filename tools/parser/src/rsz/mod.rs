use crate::layout::{Error as LayoutError, FieldKind, LayoutMap};
use crate::rsz::user::User;
use std::path::Path;

pub mod content;
pub mod user;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Rsz {
    User(User),
}

impl Rsz {
    pub fn load(path: &Path, layout: &LayoutMap) -> Result<Self> {
        let (extension, version) = path.extension_and_version()?;

        log::debug!(
            "Loading {path:?} with detected configuration: extension = {extension}, version = {version}"
        );

        let rsz = match extension.as_ref() {
            "user" => Self::User(User::load(path, &layout)?),
            _ => return Err(Error::UnrecognizedExtension(extension)),
        };

        Ok(rsz)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unrecognized file extension for file named {0}")]
    UnrecognizedExtension(String),

    #[error("path is not valid; are you sure you provided a file?")]
    InvalidPath,

    #[error("layout error: {0}")]
    Layout(#[from] LayoutError),

    #[error("invalid byte section: {0}")]
    InvalidSection(String),

    #[error("expected magic value {0:#010X} but got {1:#010X}")]
    MagicMismatch(u32, u32),

    #[error("unrecognized type id {0}")]
    UnknownLayoutId(u32),

    #[error("unexpected end of file: length is {0}, tried to read to {1}")]
    UnexpectedEof(usize, usize),

    #[error("object not found while resolving reference, index = {0}")]
    ObjectNotFound(i32),

    #[error("encountered unsupported field kind '{0:?}'")]
    UnsupportedFieldKind(FieldKind),
}

type RszExtension = String;
type RszVersion = String;

trait RszExtensionExt {
    /// Attempts to extract the RSZ document extension and version number from a source, such as
    /// a [std::path::Path]. RSZ extensions are always in the format ".<type>.<version>", e.g.
    /// ".user.3".
    fn extension_and_version(&self) -> Result<(RszExtension, RszVersion)>;
}

impl RszExtensionExt for &Path {
    fn extension_and_version(&self) -> Result<(RszExtension, RszVersion)> {
        let Some(file_name) = self
            .file_name()
            .and_then(|v| v.to_str())
            .map(|v| v.to_string())
        else {
            return Err(Error::InvalidPath);
        };

        let Some(ver) = file_name.rfind('.') else {
            return Err(Error::UnrecognizedExtension(file_name));
        };

        let Some(ext) = file_name[..ver].rfind('.') else {
            return Err(Error::UnrecognizedExtension(file_name));
        };

        Ok((
            file_name[ext + 1..ver].to_owned(),
            file_name[ver + 1..].to_owned(),
        ))
    }
}

#[macro_export]
macro_rules! check_magic {
    ($expected:expr, $actual:expr) => {{
        let expected = $expected;
        let actual = $actual;

        log::debug!(
            "Checking magic values match: expected = 0x{expected:X}, actual = 0x{actual:X}"
        );

        if expected != actual {
            return std::result::Result::Err($crate::rsz::Error::MagicMismatch(expected, actual));
        }
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;
    use std::path::Path;

    #[test]
    fn good_extension() {
        let path = Path::new("Example.user.3");
        let (extension, version) = path
            .extension_and_version()
            .expect("could not extract extension");

        assert_eq!(extension, "user");
        assert_eq!(version, "3");
    }

    #[test]
    fn bad_extension() {
        let path = Path::new("Example.user");
        let result = path.extension_and_version();

        assert_matches!(result, Err(Error::UnrecognizedExtension(_)));
    }

    #[test]
    fn no_filename() {
        let path = Path::new("some/path/");
        let result = path.extension_and_version();

        assert_matches!(result, Err(Error::InvalidPath));
    }

    #[test]
    fn empty_path() {
        let path = Path::new("");
        let result = path.extension_and_version();

        assert_matches!(result, Err(Error::InvalidPath));
    }
}
