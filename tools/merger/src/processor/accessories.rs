use crate::config::Config;
use crate::processor::{to_ingame_rarity, LanguageMap, IdMap, ReadFile, Result, Translations, WriteFile};
use crate::serde::ordered_map;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

const DATA: &str = "data/AccessoryData.json";
const TRANSLATIONS: &str = "translations/Accessory.json";

const OUTPUT: &str = "merged/Accessory.json";

pub fn process(config: &Config) -> Result {
    let data: Vec<AccessoryData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(TRANSLATIONS))?.init();

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Accessory> = Vec::with_capacity(data.len());

    for data in data {
        progress.inc(1);

        let mut accessory = Accessory::from(&data);

        for (index, lang) in translations.languages.iter().enumerate() {
            let name = translations.get(&data.name_guid, index);

            if let Some(name) = name {
                accessory.names.insert(lang.into(), name.to_owned());
            }

            let desc = translations.get(&data.description_guid, index);

            if let Some(desc) = desc {
                accessory.descriptions.insert(lang.into(), desc.to_owned());
            }
        }

        for (id, level) in data.skill_ids.iter().zip(data.skill_levels) {
            if *id != 0 {
                accessory.skills.insert(*id, level);
            }
        }

        merged.push(accessory);
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Accessory {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    rarity: u8,
    price: u16,
    level: u8,
    #[serde(serialize_with = "ordered_map")]
    skills: IdMap,
}

impl From<&AccessoryData> for Accessory {
    fn from(value: &AccessoryData) -> Self {
        Self {
            game_id: value.id,
            rarity: to_ingame_rarity(value.rarity),
            price: value.price,
            level: value.level,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            skills: IdMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AccessoryData {
    #[serde(rename = "_AccessoryId")]
    id: isize,
    #[serde(rename = "_Name")]
    name_guid: String,
    #[serde(rename = "_Explain")]
    description_guid: String,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_Price")]
    price: u16,
    #[serde(rename = "_SlotLevelAcc")]
    level: u8,
    #[serde(rename = "_Skill")]
    skill_ids: Vec<isize>,
    #[serde(rename = "_SkillLevel")]
    skill_levels: Vec<u8>,
}
