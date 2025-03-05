use crate::config::Config;
use console::Style;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;

mod accessories;
mod amulets;
mod armor;
mod charms;
mod items;
mod skills;

/// A map of RFC 639 language codes to a string value. Used to hold translations for an object
/// field.
type LanguageMap = HashMap<Language, String>;

/// A map of object IDs to a level indicator. Used for things like skill ranks granted by
/// decorations.
type IdMap = HashMap<isize, u8>;

/// A map of game IDs to an index. Used for cases where a child object needs to find its parent
/// during processing.
type LookupMap = HashMap<isize, usize>;

/// A map of game IDs to multiple indexes per-ID. Used for cases where a parent object needs to find
/// its children when those children are not stored locally.
type MultiLookupMap = HashMap<isize, Vec<usize>>;

macro_rules! _replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}

macro_rules! _count {
    ($( $tts:tt )*) => {0usize $(+ _replace_expr!($tts 1usize))*};
}

macro_rules! sections {
    (
        $( $msg:literal => $action:stmt );+
    ) => {
        let style = Style::new().dim().bold();
        let mut position = 1;
        let count = _count!($( $msg )*);

        let mut header_fn = move |message: &str| {
            println!("{} {message}", style.apply_to(format!("[{position}/{count}]")));
            position += 1;
        };

        $(
            header_fn($msg);
            $action
        )*
    };
}

pub fn all(config: &Config) -> Result {
    sections! {
        "Merging accessory files..." => accessories::process(config)?;
        "Merging item files..." => items::process(config)?;
        "Merging charm files..." => charms::process(config)?;
        "Merging amulet files..." => amulets::process(config)?;
        "Merging armor files..." => armor::process(config)?;
        "Merging skill files..." => skills::process(config)?
    }

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

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(isize)]
enum LanguageCode {
    Disabled = -1,
    Japanese,
    English,
    French,
    Italian,
    German,
    Spanish,
    Russian,
    Polish,
    Dutch,
    Portuguese,
    BrazilianPortuguese,
    Korean,
    TraditionalChinese,
    SimplifiedChinese,
    Finnish,
    Swedish,
    Danish,
    Norwegian,
    Czech,
    Hungarian,
    Slovak,
    Arabic,
    Turkish,
    Bulgarian,
    Greek,
    Romanian,
    Thai,
    Ukrainian,
    Vietnamese,
    Indonesian,
    Fiction,
    Hindi,
    LatinAmericanSpanish,
}

/// Language list from https://github.com/dtlnor/RE_MSG/blob/main/LanguagesEnum.md
#[derive(Debug, PartialEq, Eq, Deserialize, Copy, Clone, Serialize, Hash, Ord, PartialOrd)]
enum Language {
    #[serde(rename = "")]
    Disabled,
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
    #[serde(rename = "ro")]
    Romanian,
    #[serde(rename = "th")]
    Thai,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "vi")]
    Vietnamese,
    #[serde(rename = "id")]
    Indonesian,
    #[serde(skip_deserializing, rename = "")]
    Fiction,
    #[serde(rename = "hi")]
    Hindi,
    #[serde(rename = "es-419")]
    LatinAmericanSpanish,
}

impl From<&LanguageCode> for Language {
    fn from(value: &LanguageCode) -> Self {
        Self::from(*value)
    }
}

impl From<LanguageCode> for Language {
    fn from(value: LanguageCode) -> Self {
        match value {
            LanguageCode::Disabled => Self::Disabled,
            LanguageCode::Japanese => Self::Japanese,
            LanguageCode::English => Self::English,
            LanguageCode::French => Self::French,
            LanguageCode::Italian => Self::Italian,
            LanguageCode::German => Self::German,
            LanguageCode::Spanish => Self::Spanish,
            LanguageCode::Russian => Self::Russian,
            LanguageCode::Polish => Self::Polish,
            LanguageCode::Dutch => Self::Dutch,
            LanguageCode::Portuguese => Self::Portuguese,
            LanguageCode::BrazilianPortuguese => Self::BrazilianPortuguese,
            LanguageCode::Korean => Self::Korean,
            LanguageCode::TraditionalChinese => Self::TraditionalChinese,
            LanguageCode::SimplifiedChinese => Self::SimplifiedChinese,
            LanguageCode::Finnish => Self::Finnish,
            LanguageCode::Swedish => Self::Swedish,
            LanguageCode::Danish => Self::Danish,
            LanguageCode::Norwegian => Self::Norwegian,
            LanguageCode::Czech => Self::Czech,
            LanguageCode::Hungarian => Self::Hungarian,
            LanguageCode::Slovak => Self::Slovak,
            LanguageCode::Arabic => Self::Arabic,
            LanguageCode::Turkish => Self::Turkish,
            LanguageCode::Bulgarian => Self::Bulgarian,
            LanguageCode::Greek => Self::Greek,
            LanguageCode::Romanian => Self::Romanian,
            LanguageCode::Thai => Self::Thai,
            LanguageCode::Ukrainian => Self::Ukrainian,
            LanguageCode::Vietnamese => Self::Vietnamese,
            LanguageCode::Indonesian => Self::Indonesian,
            LanguageCode::Fiction => Self::Fiction,
            LanguageCode::Hindi => Self::Hindi,
            LanguageCode::LatinAmericanSpanish => Self::LatinAmericanSpanish,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Translations {
    pub languages: Vec<LanguageCode>,
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

    pub fn get(&self, guid: &str, index: usize) -> Option<&String> {
        self.find_entry(guid)?.content.get(index).and_then(|v| {
            if v.is_empty() || v == "-" || v == "---" || v.contains("#Rejected#") {
                None
            } else {
                Some(v)
            }
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde_as]
struct TranslationEntry {
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

impl<T> WriteFile for T
where
    T: Serialize,
{
    fn write_file<P: AsRef<Path>>(&self, path: P) -> Result {
        let parent = path.as_ref().parent();

        if let Some(parent) = parent {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

/// Converts an in-file rarity value to an in-game rarity value. I think.
///
/// The `_Rare` (or similar) field in the files seems to have bloated rarity values. An item with
/// an in-game rarity of 1, for example, is in the files as 18. This seems to be uniform across all
/// files with rarity values.
///
/// I don't have the brain to figure out _why_ this might be, so I'm just going to take the naive
/// way out and subtract the file value from 19 and hope I'm right that it'll be correct across the
/// board.
pub fn to_ingame_rarity(rarity: u8) -> u8 {
    19 - rarity
}

trait Lookup {
    fn find_in<'a, T>(&self, id: isize, container: &'a Vec<T>) -> Option<&'a T>;
    fn find_in_mut<'a, T>(&self, id: isize, container: &'a mut Vec<T>) -> Option<&'a mut T>;
}

impl Lookup for LookupMap {
    fn find_in<'a, T>(&self, id: isize, container: &'a Vec<T>) -> Option<&'a T> {
        if let Some(index) = self.get(&id) {
            container.get(*index)
        } else {
            None
        }
    }

    fn find_in_mut<'a, T>(&self, id: isize, container: &'a mut Vec<T>) -> Option<&'a mut T> {
        if let Some(index) = self.get(&id) {
            container.get_mut(*index)
        } else {
            None
        }
    }
}

trait MultiLookup {
    fn find_multiple_in<'a, T>(&self, id: isize, container: &'a Vec<T>) -> Vec<&'a T>;
    fn add_lookup_index(&mut self, id: isize, index: usize);
}

impl MultiLookup for MultiLookupMap {
    fn find_multiple_in<'a, T>(&self, id: isize, container: &'a Vec<T>) -> Vec<&'a T> {
        let Some(indexes) = self.get(&id) else {
            return vec![];
        };

        let mut output = Vec::with_capacity(indexes.len());

        for index in indexes {
            if let Some(item) = container.get(*index) {
                output.push(item);
            }
        }

        output
    }

    fn add_lookup_index(&mut self, id: isize, index: usize) {
        let container = if let Some(v) = self.get_mut(&id) {
            v
        } else {
            self.insert(id, Default::default());
            self.get_mut(&id).unwrap()
        };

        container.push(index);
    }
}
