use std::collections::HashMap;
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use regex::Regex;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub tools: ToolsSection,
    pub io: IoSection,
    pub extract: FilesSection,
    pub translations: FilesSection,
}

#[derive(Debug, Deserialize)]
pub struct ToolsSection {
    pub msg_extractor: PathBuf,
    pub data_extractor: PathBuf,
}

impl Default for ToolsSection {
    fn default() -> Self {
        Self {
            msg_extractor: "../REMSG_Converter/msg2json.bat".into(),
            data_extractor: "../DotUserReader/bin/Release/net8.0/DotUserReader.exe".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct IoSection {
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
pub struct FilesSection {
    pub prefix: Option<PathBuf>,
    pub files: Vec<String>,
    #[serde(default)]
    pub rules: HashMap<String, ExtractorRule>,
}

#[derive(Debug, Deserialize)]
pub struct ExtractorRule {
    #[serde(with = "serde_regex", rename = "match")]
    pub match_regex: Option<Regex>,
    pub rsz_indexes: Option<Vec<u8>>,
}

impl Default for IoSection {
    fn default() -> Self {
        Self {
            data_dir: "./data".into(),
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

    pub fn try_load<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let raw = io::read_to_string(file)?;
        Ok(toml::from_str(&raw).expect("Could not parse config file"))
    }
}
