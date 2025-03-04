use std::collections::HashMap;
use indicatif::ProgressBar;
use crate::serde::ordered_map;
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::processor::{LanguageMap, ReadFile, Result, Translations, WriteFile};

const SKILL_DATA: &str = "data/SkillCommonData.json";
const RANK_DATA: &str = "data/SkillData.json";

const SKILL_TRANSLATIONS: &str = "translations/SkillCommon.json";
const RANK_TRANSLATIONS: &str = "translations/Skill.json";

const OUTPUT: &str = "merged/Skill.json";

pub fn process(config: &Config) -> Result {
    let data: Vec<SkillData> = Vec::read_file(config.io.output_dir.join(SKILL_DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(SKILL_TRANSLATIONS))?;

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Skill> = Vec::with_capacity(data.len());
    let mut lookup: HashMap<isize, usize> = HashMap::with_capacity(data.len());

    for data in data {
        if data.id == 0 {
            continue;
        }

        progress.inc(1);

        let mut skill = Skill::from(&data);

        for (index, lang) in translations.languages.iter().enumerate() {
            if let Some(name) = translations.get_value(&data.name_guid, index) {
                skill.names.insert(*lang, name.to_owned());
            }

            if let Some(desc) = translations.get_value(&data.description_guid, index) {
                skill.descriptions.insert(*lang, desc.to_owned());
            }
        }

        lookup.insert(skill.game_id, merged.len());
        merged.push(skill);
    }

    progress.finish_and_clear();

    let data: Vec<RankData> = Vec::read_file(config.io.output_dir.join(RANK_DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(RANK_TRANSLATIONS))?;

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let Some(skill_index) = lookup.get(&data.skill_id) else {
            continue;
        };

        let mut rank = Rank::from(&data);

        for (index, lang) in translations.languages.iter().enumerate() {
            if let Some(desc) = translations.get_value(&data.description_guid, index) {
                rank.descriptions.insert(*lang, desc.to_owned());
            }
        }

        merged[*skill_index].ranks.push(rank);
    }

    progress.finish_and_clear();

    for skill in merged.iter_mut() {
        skill.ranks.sort_by_key(|v| v.level);
    }

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Skill {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    ranks: Vec<Rank>,
}

impl From<&SkillData> for Skill {
    fn from(value: &SkillData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            ranks: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Rank {
    level: u8,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
}

impl From<&RankData> for Rank {
    fn from(value: &RankData) -> Self {
        Self {
            level: value.level,
            descriptions: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SkillData {
    #[serde(rename = "_skillId")]
    id: isize,
    #[serde(rename = "_skillName")]
    name_guid: String,
    #[serde(rename = "_skillExplain")]
    description_guid: String,
}

#[derive(Debug, Deserialize)]
struct RankData {
    #[serde(rename = "_skillId")]
    skill_id: isize,
    #[serde(rename = "_SkillLv")]
    level: u8,
    #[serde(rename = "_skillExplain")]
    description_guid: String,
}
