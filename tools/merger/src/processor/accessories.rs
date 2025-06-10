use crate::placeholders::{ApplyContext, Placeholder};
use crate::processor::{
    to_ingame_rarity, IconColor, IdMap, LanguageMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const DATA: &str = "user/AccessoryData.json";
const STRINGS: &str = "msg/Accessory.json";

const OUTPUT: &str = "merged/Accessory.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Accessories);

    let data: Vec<AccessoryData> = Vec::read_file(config.io.output.join(DATA))?;
    let strings = Msg::read_file(config.io.output.join(STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Accessory> = Vec::with_capacity(data.len());

    for data in data {
        progress.inc(1);

        let mut accessory = Accessory::from(&data);

        strings.populate(&data.name_guid, &mut accessory.names);

        strings.populate(&data.description_guid, &mut accessory.descriptions);
        Placeholder::process(&mut accessory.descriptions, &ApplyContext::empty());

        for (id, level) in data.skill_ids.iter().zip(data.skill_levels) {
            if *id != 0 {
                accessory.skills.insert(*id, level);
            }
        }

        merged.push(accessory);
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
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
    allowed_on: AllowedOn,
    icon_color: IconColor,
    icon_color_id: u8,
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
            allowed_on: value.allowed_on.into(),
            icon_color: value.icon_color,
            icon_color_id: value.icon_color as u8,
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
    #[serde(rename = "_AccessoryType")]
    allowed_on: AllowedOnCode,
    #[serde(rename = "_IconColor")]
    icon_color: IconColor,
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(isize)]
enum AllowedOnCode {
    Armor = 1842954880,
    Weapon = -1638455296,
}

#[derive(Debug, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
enum AllowedOn {
    Armor,
    Weapon,
}

impl From<AllowedOnCode> for AllowedOn {
    fn from(value: AllowedOnCode) -> Self {
        match value {
            AllowedOnCode::Armor => Self::Armor,
            AllowedOnCode::Weapon => Self::Weapon,
        }
    }
}
