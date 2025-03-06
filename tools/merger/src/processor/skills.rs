use crate::config::Config;
use crate::processor::armor::{Bonus, BonusRank};
use crate::processor::{
    armor, LanguageMap, Lookup, LookupMap, ReadFile, Result, Translations, WriteFile,
};
use crate::serde::is_map_empty;
use crate::serde::ordered_map;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const SKILL_DATA: &str = "data/SkillCommonData.json";
const RANK_DATA: &str = "data/SkillData.json";

const SKILL_STRINGS: &str = "translations/SkillCommon.json";
const RANK_STRINGS: &str = "translations/Skill.json";

const OUTPUT: &str = "merged/Skill.json";

const IGNORED_IDS: &[isize] = &[
    0,
    -1950413440,
    -1724907776,
    -1702725248,
    -1577668736,
    -1540920320,
    -1478544256,
    -1437098880,
    -1203508096,
    -1196219264,
    -1110806016,
    -812084224,
    -774473472,
    -285123456,
    -111868368,
    56719788,
    309360992,
    424767232,
    457912640,
    471964960,
    504506560,
    654153152,
    1150634496,
    1406914944,
    1522720256,
    1582392192,
    1890580224,
    1960395264,
];

pub fn process(config: &Config) -> Result {
    let data: Vec<SkillData> = Vec::read_file(config.io.output_dir.join(SKILL_DATA))?;
    let strings = Translations::read_file(config.io.output_dir.join(SKILL_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Skill> = Vec::with_capacity(data.len());
    let mut lookup = LookupMap::with_capacity(data.len());

    for data in data {
        if IGNORED_IDS.contains(&data.id) {
            continue;
        }

        progress.inc(1);

        let mut skill = Skill::from(&data);

        for (index, lang) in strings.languages.iter().enumerate() {
            if let Some(name) = strings.get(&data.name_guid, index) {
                skill.names.insert(lang.into(), name.to_owned());
            }

            if let Some(desc) = strings.get(&data.description_guid, index) {
                skill.descriptions.insert(lang.into(), desc.to_owned());
            }
        }

        lookup.insert(skill.game_id, merged.len());
        merged.push(skill);
    }

    progress.finish_and_clear();

    let data: Vec<RankData> = Vec::read_file(config.io.output_dir.join(RANK_DATA))?;
    let strings = Translations::read_file(config.io.output_dir.join(RANK_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        if IGNORED_IDS.contains(&data.skill_id) {
            continue;
        }

        let skill = lookup
            .find_in_mut(data.skill_id, &mut merged)
            .expect(&format!("Could not find skill {}", data.skill_id));

        let mut rank = Rank::from(&data);

        for (index, lang) in strings.languages.iter().enumerate() {
            if let Some(name) = strings.get(&data.name_guid, index) {
                rank.names.insert(lang.into(), name.to_owned());
            }

            if let Some(desc) = strings.get(&data.description_guid, index) {
                rank.descriptions.insert(lang.into(), desc.to_owned());
            }
        }

        skill.ranks.push(rank);
    }

    for skill in merged.iter_mut() {
        skill.ranks.sort_by_key(|v| v.level);

        // Set bonus skills encode the number of pieces required for the bonus as the skill level.
        // Once sorted, we can convert that to a "real" level by setting the level to the index + 1.
        if skill.kind.is_armor_bonus() {
            for (index, rank) in skill.ranks.iter_mut().enumerate() {
                rank.level = (index as u8) + 1;
            }
        }
    }

    progress.finish_and_clear();

    let mut data: Vec<armor::Set> = Vec::read_file(config.io.output_dir.join(armor::OUTPUT))?;
    let progress = ProgressBar::new(data.len() as u64);

    for data in data.iter_mut() {
        progress.inc(1);

        let Some(armor) = data.pieces.first() else {
            continue;
        };

        for id in armor.skills.keys().copied() {
            let Some(skill) = lookup.find_in(id, &mut merged) else {
                continue;
            };

            let bonus = match skill.kind {
                SkillKind::Group => &mut data.group_bonus,
                SkillKind::Set => &mut data.set_bonus,
                _ => continue,
            };

            let bonus = bonus.get_or_insert_with(|| Bonus {
                skill_id: id,
                ranks: Vec::new(),
            });

            for rank in &skill.ranks {
                bonus.ranks.push(BonusRank {
                    pieces: rank.pieces,
                    skill_level: rank.level,
                });
            }

            bonus.ranks.sort_by_key(|v| v.pieces);
        }
    }

    data.write_file(config.io.output_dir.join(armor::OUTPUT))?;

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Skill {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map", skip_serializing_if = "is_map_empty")]
    descriptions: LanguageMap,
    ranks: Vec<Rank>,
    kind: SkillKind,
}

impl From<&SkillData> for Skill {
    fn from(value: &SkillData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            ranks: Vec::new(),
            kind: value.kind,
        }
    }
}

#[derive(Debug, Serialize)]
struct Rank {
    level: u8,
    // Used for set and group bonuses, not serialized to merged file.
    #[serde(skip)]
    pieces: u8,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    #[serde(skip_serializing_if = "is_map_empty", serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&RankData> for Rank {
    fn from(value: &RankData) -> Self {
        Self {
            level: value.level,
            pieces: value.level,
            names: LanguageMap::new(),
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
    #[serde(rename = "_skillCategory")]
    kind: SkillKind,
}

#[derive(Debug, Deserialize)]
struct RankData {
    #[serde(rename = "_skillId")]
    skill_id: isize,
    #[serde(rename = "_SkillLv")]
    level: u8,
    #[serde(rename = "_skillName")]
    name_guid: String,
    #[serde(rename = "_skillExplain")]
    description_guid: String,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
#[repr(u8)]
pub enum SkillKind {
    Armor,
    Set,
    Group,
    Weapon,
}

impl SkillKind {
    fn is_armor_bonus(&self) -> bool {
        matches!(self, Self::Set | Self::Group)
    }
}
