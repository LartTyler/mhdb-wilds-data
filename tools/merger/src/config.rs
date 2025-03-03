use std::fs::File;
use std::io;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub io: IoSection,
}

#[derive(Debug, Deserialize)]
pub struct IoSection {
    pub output_dir: PathBuf,
}

impl Default for IoSection {
    fn default() -> Self {
        Self {
            output_dir: "./output".into(),
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Self {
        if let Some(path) = path {
            Self::try_load(path).expect("Could not load config file")
        } else {
            Self::try_load("config.toml").unwrap_or_default()
        }
    }

    pub fn try_load<P>(path: P) -> io::Result<Self> where P: AsRef<Path> {
        let file = File::open(path)?;
        let raw = io::read_to_string(file)?;
        Ok(toml::from_str(&raw).expect("Could not parse config file"))
    }
}
