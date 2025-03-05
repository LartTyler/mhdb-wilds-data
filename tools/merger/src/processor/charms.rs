use crate::config::Config;
use crate::processor::{LanguageMap, ReadFile, Result, Translations, WriteFile};
use crate::serde::ordered_map;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

const DATA: &str = "data/Charm.json";
const TRANSLATIONS: &str = "translations/Charm.json";

const OUTPUT: &str = "merged/Charm.json";

pub fn process(config: &Config) -> Result {
    let data: Vec<CharmData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(TRANSLATIONS))?;

    let mut merged: Vec<Charm> = Vec::with_capacity(data.len());
    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let mut charm = Charm::from(&data);

        for (index, lang) in translations.languages.iter().enumerate() {
            if let Some(name) = translations.get(&data.name_guid, index) {
                charm.names.insert(lang.into(), name.to_owned());
            }
        }

        merged.push(charm);
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Charm {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&CharmData> for Charm {
    fn from(value: &CharmData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CharmData {
    #[serde(rename = "_Type")]
    id: isize,
    #[serde(rename = "_Name")]
    name_guid: String,
}
