use regex::Regex;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub tools: Tools,
    pub io: Io,
    pub user: Files,
    pub msg: Files,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Self {
        let config_path = path.unwrap_or_else(|| Path::new("config.toml"));

        match Self::try_load(config_path) {
            Ok(Some(v)) => v,
            Ok(None) => {
                // If load() was called with a path, and we got None back from try_load(), the file
                // didn't exist and we should panic.
                if path.is_some() {
                    panic!("Config file does not exist: {path:?}");
                }

                Self::default()
            }
            Err(e) => panic!("Could not load config file: {e}"),
        }
    }

    pub fn try_load<P: AsRef<Path>>(path: P) -> Result<Option<Self>, Error> {
        let path = path.as_ref();

        if !fs::exists(path)? {
            return Ok(None);
        }

        let file = File::open(path)?;
        let raw = io::read_to_string(file)?;
        Ok(toml::from_str(&raw)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct Tools {
    pub user: PathBuf,
    pub msg: PathBuf,
}

impl Default for Tools {
    fn default() -> Self {
        Self {
            user: PathBuf::from("tools/DotUserReader/bin/Release/net8.0/DotUserReader.exe"),
            msg: PathBuf::from("tools/REMSG_Converter/REMSG_Converter.exe"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Io {
    pub data: Vec<PathBuf>,
    pub output: PathBuf,
}

impl Default for Io {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            output: PathBuf::from("output"),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Files {
    pub input_prefix: Option<PathBuf>,
    pub targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
pub struct Target {
    pub files: Vec<String>,
    pub output_prefix: Option<PathBuf>,

    #[serde(default)]
    pub transform: Vec<Transform>,
}

impl Target {
    pub fn find_transform<P: AsRef<str>>(&self, path: P) -> Option<&Transform> {
        self.transform.iter().find(|v| v.matches(&path))
    }
}

#[derive(Debug, Deserialize)]
pub struct Transform {
    #[serde(rename = "match", with = "serde_regex")]
    pub pattern: Regex,

    #[serde(default)]
    pub rsz: Vec<u8>,
}

impl Transform {
    pub fn matches<P: AsRef<str>>(&self, path: P) -> bool {
        self.pattern.is_match(path.as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] io::Error),

    #[error("toml: {0}")]
    Toml(#[from] toml::de::Error),
}
