use crate::config::Config;
use crate::processor::{LanguageMap, LevelMap, ReadFile, Result, Translations, WriteFile};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DATA: &str = "data/AccessoryData.json";
const TRANSLATIONS: &str = "translations/Accessory.json";

const OUTPUT: &str = "merged/Accessory.json";

pub fn process(config: &Config) -> Result {
    let data: Vec<AccessoryData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(TRANSLATIONS))?.init();

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Accessory> = Vec::with_capacity(data.len());

    for item in data {
        progress.inc(1);

        let mut names = LanguageMap::new();
        let mut descriptions = LanguageMap::new();

        for (index, lang) in translations.languages.iter().enumerate() {
            let name = translations.get_value(&item.name_guid, index);

            if let Some(name) = name {
                names.insert(*lang, name.to_owned());
            }

            let desc = translations.get_value(&item.description_guid, index);

            if let Some(desc) = desc {
                descriptions.insert(*lang, desc.to_owned());
            }
        }

        let mut skills = LevelMap::new();

        for (id, level) in item.skill_ids.iter().zip(item.skill_levels) {
            if *id != 0 {
                skills.insert(*id, level);
            }
        }

        merged.push(Accessory {
            game_id: item.id,
            rarity: item.rarity,
            price: item.price,
            level: item.level,
            names,
            descriptions,
            skills,
        });
    }

    progress.finish_and_clear();

    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Accessory {
    game_id: isize,
    names: LanguageMap,
    descriptions: LanguageMap,
    rarity: u8,
    price: u16,
    level: u8,
    skills: HashMap<isize, u8>,
}

#[derive(Debug, Deserialize)]
struct AccessoryData {
    #[serde(rename = "_Index")]
    index: isize,
    #[serde(rename = "_AccessoryId")]
    id: isize,
    #[serde(rename = "_Name")]
    name_guid: String,
    #[serde(rename = "_Explain")]
    description_guid: String,
    #[serde(rename = "_AccessoryType")]
    kind_id: isize,
    #[serde(rename = "_SortId")]
    sort_id: isize,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_IconColor")]
    icon_color: u8,
    #[serde(rename = "_Price")]
    price: u16,
    #[serde(rename = "_SlotLevelAcc")]
    level: u8,
    #[serde(rename = "_Skill")]
    skill_ids: Vec<isize>,
    #[serde(rename = "_SkillLevel")]
    skill_levels: Vec<u8>,
}
