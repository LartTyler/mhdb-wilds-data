use std::collections::HashMap;
use std::fs;
use crate::config::Config;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize, Serializer};
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use std::fs::File;
use std::io::Write;
use std::path::Path;

mod accessories;

pub fn all(config: &Config) -> Result {
    accessories::process(config)?;

    Ok(())
}

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse: {0}")]
    Parse(#[from] serde_json::Error),
}

/// Language list from https://github.com/dtlnor/RE_MSG/blob/main/LanguagesEnum.md
#[derive(Debug, PartialEq, Eq, Deserialize_repr, Copy, Clone, Serialize, Hash)]
#[repr(isize)]
pub enum Language {
    #[serde(rename = "")]
    Disabled = -1,
    #[serde(rename = "jp")]
    Japanese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "nl")]
    Dutch,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "pt-BR")]
    BrazilianPortuguese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "zh-Hant")]
    TraditionalChinese,
    #[serde(rename = "zh-Hans")]
    SimplifiedChinese,
    #[serde(rename = "fi")]
    Finnish,
    #[serde(rename = "sv")]
    Swedish,
    #[serde(rename = "da")]
    Danish,
    #[serde(rename = "no")]
    Norwegian,
    #[serde(rename = "cs")]
    Czech,
    #[serde(rename = "hu")]
    Hungarian,
    #[serde(rename = "sk")]
    Slovak,
    #[serde(rename = "ar")]
    Arabic,
    #[serde(rename = "tr")]
    Turkish,
    #[serde(rename = "bg")]
    Bulgarian,
    #[serde(rename = "el")]
    Greek,

    Romanian,
    Thai,
    Ukrainian,
    Vietnamese,
    Indonesian,
    #[serde(rename = "")]
    Fiction,
    Hindi,
    #[serde(rename = "es-419")]
    LatinAmericanSpanish,
}

#[derive(Debug, Deserialize)]
pub struct Translations {
    pub languages: Vec<Language>,
    pub entries: Vec<TranslationEntry>,
    #[serde(skip)]
    pub guid_map: HashMap<String, usize>,
}

impl Translations {
    pub fn init(mut self) -> Self {
        for (index, entry) in self.entries.iter().enumerate() {
            self.guid_map.insert(entry.guid.to_owned(), index);
        }

        self
    }

    pub fn find_entry(&self, guid: &str) -> Option<&TranslationEntry> {
        if !self.guid_map.is_empty() {
            let index = self.guid_map.get(guid)?;
            self.entries.get(*index)
        } else {
            for entry in &self.entries {
                if entry.guid == guid {
                    return Some(entry);
                }
            }

            None
        }
    }

    pub fn get_value(&self, guid: &str, index: usize) -> Option<&String> {
        self.find_entry(guid)?.content.get(index)
    }
}

#[derive(Debug, Deserialize)]
#[serde_as]
pub struct TranslationEntry {
    pub guid: String,

    #[serde_as(as = "Vec<NoneAsEmptyString>")]
    pub content: Vec<String>,
}

trait ReadFile {
    fn read_file<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}

impl<T> ReadFile for T
where
    T: Sized + DeserializeOwned,
{
    fn read_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

trait WriteFile {
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result;
}

impl<T> WriteFile for T where T: Serialize {
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result {
        let parent = path.as_ref().parent();

        if let Some(parent) = parent {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, serde_json::to_string(self)?)?;
        Ok(())
    }
}
