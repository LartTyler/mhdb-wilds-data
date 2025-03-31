use serde::Deserialize;
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
}

impl Msg {
    pub fn get_language_index(&self, language: LanguageCode) -> Option<usize> {
        self.languages.iter().position(|v| *v == language)
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

#[derive(Debug, Deserialize_repr, Copy, Clone, Eq, PartialEq)]
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
