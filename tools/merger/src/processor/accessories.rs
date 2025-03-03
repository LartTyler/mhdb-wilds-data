use crate::config::Config;
use crate::processor::{Language, ReadFile, Result, Translations, WriteFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DATA: &str = "data/AccessoryData.json";
const TRANSLATIONS: &str = "translations/Accessory.json";

type LanguageMap = HashMap<Language, String>;
type SkillMap = HashMap<isize, u8>;

pub fn process(config: &Config) -> Result {
    let data: Vec<AccessoryData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(TRANSLATIONS))?.init();

    let mut merged: Vec<Accessory> = Vec::with_capacity(data.len());

    for item in data {
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

        let mut skills = SkillMap::new();

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

    merged.write_file(config.io.output_dir.join("merged").join("Accessory.json"))
}

#[derive(Debug, Serialize)]
struct Accessory {
    pub game_id: isize,
    pub names: LanguageMap,
    pub descriptions: LanguageMap,
    pub rarity: u8,
    pub price: u16,
    pub level: u8,
    pub skills: HashMap<isize, u8>,
}

#[derive(Debug, Deserialize)]
struct AccessoryData {
    #[serde(rename = "_Index")]
    pub index: isize,

    #[serde(rename = "_AccessoryId")]
    pub id: isize,

    #[serde(rename = "_Name")]
    pub name_guid: String,

    #[serde(rename = "_Explain")]
    pub description_guid: String,

    #[serde(rename = "_AccessoryType")]
    pub kind_id: isize,

    #[serde(rename = "_SortId")]
    pub sort_id: isize,

    #[serde(rename = "_Rare")]
    pub rarity: u8,

    #[serde(rename = "_IconColor")]
    pub icon_color: u8,

    #[serde(rename = "_Price")]
    pub price: u16,

    #[serde(rename = "_SlotLevelAcc")]
    pub level: u8,

    #[serde(rename = "_Skill")]
    pub skill_ids: Vec<isize>,

    #[serde(rename = "_SkillLevel")]
    pub skill_levels: Vec<u8>,
}
