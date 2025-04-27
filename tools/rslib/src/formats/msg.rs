use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use serde_with::serde_as;
use std::cell::OnceCell;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Msg {
    pub languages: Vec<LanguageCode>,
    pub entries: Vec<MsgEntry>,
    #[serde(skip)]
    guid_map: OnceCell<HashMap<String, usize>>,
    #[serde(skip)]
    name_map: OnceCell<HashMap<String, usize>>,
    #[serde(skip)]
    lang_map: OnceCell<HashMap<LanguageCode, usize>>,
}

impl Msg {
    pub fn get_language_index(&self, language: LanguageCode) -> Option<usize> {
        let lookup = self.lang_map.get_or_init(|| {
            self.languages
                .iter()
                .enumerate()
                .map(|(index, v)| (*v, index))
                .collect()
        });

        lookup.get(&language).cloned()
    }

    pub fn find(&self, guid: &str) -> Option<&MsgEntry> {
        let lookup = self.guid_map.get_or_init(|| {
            self.entries
                .iter()
                .enumerate()
                .map(|(index, v)| (v.guid.to_owned(), index))
                .collect()
        });

        self.entries.get(*lookup.get(guid)?)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&MsgEntry> {
        let lookup = self.name_map.get_or_init(|| {
            self.entries
                .iter()
                .enumerate()
                .map(|(index, v)| (v.name.to_owned(), index))
                .collect()
        });

        self.entries.get(*lookup.get(name)?)
    }

    pub fn find_lang_by_name(&self, name: &str, lang: LanguageCode) -> Option<&str> {
        let index = self.get_language_index(lang)?;
        self.find_by_name(name)?.get(index)
    }

    pub fn get(&self, guid: &str, index: usize) -> Option<&str> {
        self.find(guid)?.get(index)
    }

    pub fn get_lang(&self, guid: &str, lang: LanguageCode) -> Option<&str> {
        let index = self.get_language_index(lang)?;
        self.get(guid, index)
    }

    pub fn get_by_name(&self, name: &str, index: usize) -> Option<&str> {
        self.find_by_name(name)?.get(index)
    }
}

#[derive(Debug, Deserialize)]
#[serde_as]
pub struct MsgEntry {
    pub name: String,
    pub guid: String,
    #[serde_as(as = "Vec<NoneAsEmptyString>")]
    pub content: Vec<String>,
}

impl MsgEntry {
    pub fn get(&self, index: usize) -> Option<&str> {
        let item = self.content.get(index)?;

        if item.is_empty() || item == "-" || item == "---" || item.contains("#Rejected#") {
            None
        } else {
            Some(item.as_ref())
        }
    }
}

#[derive(Debug, Deserialize_repr, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(isize)]
pub enum LanguageCode {
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

/// A map of RFC 639 language codes to a string value. Used to hold translations for an object
/// field.
pub type LanguageMap = HashMap<Language, String>;

/// Language list from https://github.com/dtlnor/RE_MSG/blob/main/LanguagesEnum.md
#[derive(Debug, PartialEq, Eq, Deserialize, Copy, Clone, Serialize, Hash, Ord, PartialOrd)]
pub enum Language {
    #[serde(rename = "")]
    Disabled,
    #[serde(rename = "ja")]
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

impl From<Language> for LanguageCode {
    fn from(value: Language) -> Self {
        match value {
            Language::Disabled => LanguageCode::Disabled,
            Language::Japanese => LanguageCode::Japanese,
            Language::English => LanguageCode::English,
            Language::French => LanguageCode::French,
            Language::Italian => LanguageCode::Italian,
            Language::German => LanguageCode::German,
            Language::Spanish => LanguageCode::Spanish,
            Language::Russian => LanguageCode::Russian,
            Language::Polish => LanguageCode::Polish,
            Language::Dutch => LanguageCode::Dutch,
            Language::Portuguese => LanguageCode::Portuguese,
            Language::BrazilianPortuguese => LanguageCode::BrazilianPortuguese,
            Language::Korean => LanguageCode::Korean,
            Language::TraditionalChinese => LanguageCode::TraditionalChinese,
            Language::SimplifiedChinese => LanguageCode::SimplifiedChinese,
            Language::Finnish => LanguageCode::Finnish,
            Language::Swedish => LanguageCode::Swedish,
            Language::Danish => LanguageCode::Danish,
            Language::Norwegian => LanguageCode::Norwegian,
            Language::Czech => LanguageCode::Czech,
            Language::Hungarian => LanguageCode::Hungarian,
            Language::Slovak => LanguageCode::Slovak,
            Language::Arabic => LanguageCode::Arabic,
            Language::Turkish => LanguageCode::Turkish,
            Language::Bulgarian => LanguageCode::Bulgarian,
            Language::Greek => LanguageCode::Greek,
            Language::Romanian => LanguageCode::Romanian,
            Language::Thai => LanguageCode::Thai,
            Language::Ukrainian => LanguageCode::Ukrainian,
            Language::Vietnamese => LanguageCode::Vietnamese,
            Language::Indonesian => LanguageCode::Indonesian,
            Language::Fiction => LanguageCode::Fiction,
            Language::Hindi => LanguageCode::Hindi,
            Language::LatinAmericanSpanish => LanguageCode::LatinAmericanSpanish,
        }
    }
}

impl From<&Language> for LanguageCode {
    fn from(value: &Language) -> Self {
        (*value).into()
    }
}
