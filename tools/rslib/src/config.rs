use regex::Regex;
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub tools: ToolsSection,
    pub io: IoSection,
    pub user: FilesSection,
    pub msg: FilesSection,
}

#[derive(Debug, Deserialize)]
pub struct ToolsSection {
    pub msg_extractor: PathBuf,
    pub user_extractor: PathBuf,
}

impl Default for ToolsSection {
    fn default() -> Self {
        Self {
            msg_extractor: "tools/REMSG_Converter/msg2json.bat".into(),
            user_extractor: "tools/DotUserReader/bin/Release/net8.0/DotUserReader.exe".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IoSection {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl Default for IoSection {
    fn default() -> Self {
        Self {
            data_dir: "./data".into(),
            output_dir: "./output".into(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FilesSection {
    pub prefix: Option<PathBuf>,
    pub files: Vec<String>,
    #[serde(default)]
    pub rules: Vec<ExtractorRule>,
}

impl FilesSection {
    pub fn get_matching_rule<S: AsRef<str>>(&self, path: S) -> Option<&ExtractorRule> {
        self.rules.iter().find(|&rule| rule.matches(&path))
    }
}

#[derive(Debug, Deserialize)]
pub struct ExtractorRule {
    #[serde(with = "serde_regex", rename = "match")]
    pub match_regex: Option<Regex>,
    #[serde(default)]
    pub rsz_indexes: Vec<u8>,
}

impl ExtractorRule {
    pub fn matches<S: AsRef<str>>(&self, path: S) -> bool {
        match &self.match_regex {
            Some(regex) => regex.is_match(path.as_ref()),
            None => true,
        }
    }
}

impl Config {
    pub fn load(path: Option<&Path>) -> Self {
        if let Some(path) = path {
            match Self::try_load(path) {
                Ok(Some(v)) => v,
                Ok(None) => panic!("Config file does not exist: {:?}", path),
                Err(e) => panic!("Could not load config file: {}", e),
            }
        } else {
            match Self::try_load("config.toml") {
                Ok(Some(v)) => v,
                Ok(None) => Self::default(),
                Err(e) => panic!("Could not load config file: {}", e),
            }
        }
    }

    pub fn try_load<P: AsRef<Path>>(path: P) -> Result<Option<Self>> {
        let path = path.as_ref();

        if !fs::exists(path)? {
            return Ok(None);
        }

        let file = File::open(path)?;
        let raw = io::read_to_string(file)?;
        Ok(toml::from_str(&raw)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
}
