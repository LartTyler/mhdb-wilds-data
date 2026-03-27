use crate::placeholders::{ApplyContext, Placeholder};
use crate::processor::armor::{Bonus, BonusRank};
use crate::processor::{
    armor, LanguageMap, Lookup, LookupMap, PopulateStrings, Processor, ReadFile, Result, WriteFile,
};
use crate::serde::is_default;
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const SKILL_DATA: &str = "user/SkillCommonData.json";
const RANK_DATA: &str = "user/SkillData.json";

const SKILL_STRINGS: &str = "msg/SkillCommon.json";
const RANK_STRINGS: &str = "msg/Skill.json";

const OUTPUT: &str = "merged/Skill.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Skill);

    let data: Vec<SkillData> = Vec::read_file(config.io.output.join(SKILL_DATA))?;
    let strings = Msg::read_file(config.io.output.join(SKILL_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Skill> = Vec::with_capacity(data.len());
    let mut lookup = LookupMap::with_capacity(data.len());

    let placeholder_context = ApplyContext::empty();

    for data in data {
        progress.inc(1);

        let mut skill = Skill::from(&data);

        strings.populate(&data.name_guid, &mut skill.names);

        // Ignore skills with no names
        if skill.names.is_empty() {
            continue;
        }

        strings.populate(&data.description_guid, &mut skill.descriptions);
        Placeholder::process(&mut skill.descriptions, &placeholder_context);

        lookup.insert(skill.game_id, merged.len());
        merged.push(skill);
    }

    progress.finish_and_clear();

    let data: Vec<RankData> = Vec::read_file(config.io.output.join(RANK_DATA))?;
    let strings = Msg::read_file(config.io.output.join(RANK_STRINGS))?;

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let Some(skill) = lookup.find_in_mut(data.skill_id, &mut merged) else {
            // This is almost certainly a skill rank for a skill we skipped for not having any names
            // set in the translations file.
            continue;
        };

        let mut rank = Rank::from(&data);

        strings.populate(&data.name_guid, &mut rank.names);

        strings.populate(&data.description_guid, &mut rank.descriptions);
        Placeholder::process(&mut rank.descriptions, &placeholder_context);

        skill.ranks.push(rank);
    }

    for skill in merged.iter_mut() {
        skill.ranks.sort_by_key(|v| v.level);

        // Set bonus skills encode the number of pieces required for the bonus as the skill level.
        // Once sorted, we can convert that to a "real" level by setting the level to the index + 1.
        if skill.kind.is_armor_bonus() {
            for (index, rank) in skill.ranks.iter_mut().enumerate() {
                rank.set_pieces_required = rank.level;
                rank.level = (index as u8) + 1;
            }
        }
    }

    progress.finish_and_clear();

    let mut data: Vec<armor::Set> = Vec::read_file(config.io.output.join(armor::OUTPUT))?;
    let progress = ProgressBar::new(data.len() as u64);

    for data in &mut data {
        progress.inc(1);

        // We only need to consider the first piece of armor in the set, since every piece should
        // have the same set/group bonuses (except for "cross-series" bonuses, such as the
        // Gogmazios Alpha Helm also counting as one piece for Zoh Shia's Pulse)
        let Some(armor) = data.pieces.first_mut() else {
            continue;
        };

        for id in armor.skills.keys().copied() {
            let Some(skill) = lookup.find_in(id, &merged) else {
                continue;
            };

            let bonus_slot = match skill.kind {
                SkillKind::Group => &mut data.group_bonus_id,
                SkillKind::Set => &mut data.set_bonus_id,
                _ => continue,
            };

            // Only the first entry belonging to the matched slot should be persisted. Additional
            // entries of the same type are "cross-series" bonuses.
            if bonus_slot.is_none() {
                *bonus_slot = Some(skill.game_id);
            }

            // TODO Deprecated
            // region Deprecated: To be removed after 2026-05-01
            let deprecated_bonus = match skill.kind {
                SkillKind::Group => &mut data.group_bonus,
                SkillKind::Set => &mut data.set_bonus,
                _ => continue,
            };

            let deprecated_bonus = deprecated_bonus.get_or_insert_with(|| Bonus {
                skill_id: id,
                ranks: Vec::new(),
            });

            for rank in &skill.ranks {
                deprecated_bonus.ranks.push(BonusRank {
                    pieces: rank.set_pieces_required,
                    skill_level: rank.level,
                });
            }

            deprecated_bonus.ranks.sort_by_key(|v| v.pieces);
            // endregion
        }
    }

    data.write_file(config.io.output.join(armor::OUTPUT))?;

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Skill {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(
        serialize_with = "ordered_map",
        skip_serializing_if = "LanguageMap::is_empty"
    )]
    descriptions: LanguageMap,
    ranks: Vec<Rank>,
    kind: SkillKind,
    icon: IconKind,
    icon_id: u8,
}

impl From<&SkillData> for Skill {
    fn from(value: &SkillData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            ranks: Vec::new(),
            kind: value.kind,
            icon: value.icon,
            icon_id: value.icon as u8,
        }
    }
}

#[derive(Debug, Serialize)]
struct Rank {
    level: u8,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    #[serde(
        skip_serializing_if = "LanguageMap::is_empty",
        serialize_with = "ordered_map"
    )]
    names: LanguageMap,
    #[serde(skip_serializing_if = "is_default")]
    set_pieces_required: u8,
}

impl From<&RankData> for Rank {
    fn from(value: &RankData) -> Self {
        Self {
            level: value.level,
            set_pieces_required: 0,
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
    #[serde(rename = "_SkillIconType")]
    icon: IconKind,
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

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "kebab-case"))]
#[repr(u8)]
pub enum IconKind {
    None = 0,
    Attack = 1,
    Affinity = 2,
    Element = 3,
    Handicraft = 4,
    Ranged = 5,
    Defense = 6,
    Health = 7,
    Stamina = 8,
    Offense = 9,
    Utility = 10,
    Item = 11,
    Gathering = 12,
    Group = 13,
    Set = 14,
}
