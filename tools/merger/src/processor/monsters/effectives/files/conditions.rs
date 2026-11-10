use crate::processor::monsters::effectives::{AttributeKind, EffectKind};
use crate::processor::monsters::{Monster, MonsterId};
use crate::processor::weapons::{Element, Status};
use crate::processor::{FileObjects, Guid, LanguageMap, PopulateStrings, ReadFile};
use macros::add_condition;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

const PRESET_DATA: &str = "user/monsters/EmParamBadConditionPreset.json";
const CONDITIONS_DATA: &str = "user/monsters/EmParamBadCondition2.json";
const CONDITION_TEXT_DATA: &str = "user/monsters/EnemyReportMeasureFreeInfoData.json";
const CONDITION_TEXT_STRINGS: &str = "msg/EnemyReportMeasureFreeInfoText.json";

pub fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    let presets = ConditionPresetTables::read_file(config.data_path(PRESET_DATA))?;
    let data: Vec<MonsterConditions> = Vec::read_file(config.data_path(CONDITIONS_DATA))?;

    for data in data {
        let Some(monster) = monsters.get_mut(data.monster_id) else {
            log::debug!(
                "Skipping conditions for {}, monster not found",
                data.monster_id
            );
            continue;
        };

        add_condition!(presets.paralyze, data.paralyze => monster, Status::Paralysis);
        add_condition!(presets.poison, data.poison => monster, Status::Poison);
        add_condition!(presets.sleep, data.sleep => monster, Status::Sleep);
        add_condition!(presets.stun, data.stun => monster, Effect::Stun);
        add_condition!(presets.flash, data.flash => monster, Effect::Flash);
        add_condition!(presets.noise, data.noise => monster, Effect::Noise);
        add_condition!(presets.blast, data.blast => monster, Status::Blastblight);
        add_condition!(presets.exhaust, data.exhaust => monster, Effect::Exhaust);
    }

    let data: Vec<ConditionText> = Vec::read_file(config.data_path(CONDITION_TEXT_DATA))?;
    let strings = Msg::read_file(config.data_path(CONDITION_TEXT_STRINGS))?;

    for data in data {
        let Some(monster) = monsters.get_mut(data.monster_id) else {
            log::debug!(
                "Skipping condition text for {}, monster not found",
                data.monster_id
            );
            continue;
        };

        let search = AttributeKind::from(data.kind);
        let Some(weakness) = monster.weaknesses.iter_mut().find(|v| v.kind == search) else {
            log::debug!(
                "Skipping condition text for {}, monster does not have a {search:?} weakness",
                data.monster_id,
            );

            continue;
        };

        let mut values = LanguageMap::new();
        strings.populate(&data.guid, &mut values);

        weakness.condition = Some(values);
    }

    for monster in monsters.items_mut() {
        monster.resistances.sort_by_key(|v| v.kind);
        monster.weaknesses.sort_by_key(|v| v.kind);
    }

    Ok(())
}

type PresetMap = HashMap<Guid, ConditionPreset>;

#[derive(Debug, Deserialize)]
struct ConditionPresetTables {
    #[serde(rename = "_Paralyze", deserialize_with = "vec_to_map")]
    paralyze: PresetMap,
    #[serde(rename = "_Poison", deserialize_with = "vec_to_map")]
    poison: PresetMap,
    #[serde(rename = "_Sleep", deserialize_with = "nested_vec_to_map")]
    sleep: PresetMap,
    #[serde(rename = "_Stun", deserialize_with = "nested_vec_to_map")]
    stun: PresetMap,
    #[serde(rename = "_Blast", deserialize_with = "vec_to_map")]
    blast: PresetMap,
    #[serde(rename = "_Stamina", deserialize_with = "vec_to_map")]
    exhaust: PresetMap,
    #[serde(rename = "_Ear", deserialize_with = "nested_vec_to_map")]
    noise: PresetMap,
    #[serde(rename = "_Flash", deserialize_with = "nested_vec_to_map")]
    flash: PresetMap,
}

fn vec_to_map<'de, D>(deserializer: D) -> Result<PresetMap, D::Error>
where
    D: Deserializer<'de>,
{
    let contents: Vec<ConditionPreset> = Vec::deserialize(deserializer)?;
    let map = contents
        .into_iter()
        .map(|v| (Guid(v.id.clone()), v))
        .collect();

    Ok(map)
}

fn nested_vec_to_map<'de, D>(deserializer: D) -> Result<PresetMap, D::Error>
where
    D: Deserializer<'de>,
{
    let nested = NestedPresetMap::deserialize(deserializer)?;
    Ok(nested.contents)
}

#[derive(Debug, Deserialize)]
struct NestedPresetMap {
    #[serde(rename = "_PresetArray", deserialize_with = "vec_to_map")]
    contents: PresetMap,
}

#[derive(Debug, Deserialize)]
struct ConditionPreset {
    #[serde(rename = "_InstanceGuid")]
    id: Guid,
    #[serde(rename = "_EffectiveType")]
    tier: DamageTier,
}

#[derive(Debug, Deserialize_repr)]
#[repr(isize)]
enum DamageTier {
    Disabled = -1152996608,
    Enabled = 1294789760,
    One = 1693907968,
    Two = -1937674624,
    Three = -1279992448,
}

type DamageTierValue = u8;

impl DamageTier {
    fn as_number(&self) -> DamageTierValue {
        match self {
            Self::Disabled => 0,
            Self::Enabled | Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
        }
    }
}

#[derive(Debug, Deserialize)]
struct MonsterConditions {
    #[serde(rename = "_EmId")]
    monster_id: MonsterId,
    #[serde(rename = "ParalyzePriset")]
    paralyze: Guid,
    #[serde(rename = "PoisonPriset")]
    poison: Guid,
    #[serde(rename = "SleepPriset")]
    sleep: Guid,
    #[serde(rename = "StunPriset")]
    stun: Guid,
    #[serde(rename = "FlashPriset")]
    flash: Guid,
    #[serde(rename = "BlastPreset")]
    blast: Guid,
    #[serde(rename = "StaminaPreset")]
    exhaust: Guid,
    #[serde(rename = "EarPriset")]
    noise: Guid,
}

#[derive(Debug, Deserialize)]
struct ConditionText {
    #[serde(rename = "_EmID")]
    monster_id: MonsterId,
    #[serde(rename = "_DispType")]
    kind: WeaknessConditionKind,
    #[serde(rename = "_FreeInfo")]
    guid: Guid,
}

#[derive(Debug, Deserialize_repr, Eq, PartialEq)]
#[repr(u8)]
enum WeaknessConditionKind {
    Thunder = 2,
    Noise = 12,
}

impl From<WeaknessConditionKind> for AttributeKind {
    fn from(value: WeaknessConditionKind) -> Self {
        match value {
            WeaknessConditionKind::Thunder => Element::Thunder.into(),
            WeaknessConditionKind::Noise => EffectKind::Noise.into(),
        }
    }
}

mod macros {
    macro_rules! add_condition {
        ($presets:expr , $guid:expr => $monster:ident , $enum:ident :: $variant:ident) => {
            if $guid.is_empty() {
                add_condition!(@ $monster resist $enum::$variant);
            } else {
                #[allow(unused)]
                let Some(preset) = $presets.get(&$guid) else {
                    panic!("Could not find entry for monster {}", $monster.game_id);
                };

                add_condition!(@ preset => $monster weak $enum::$variant);
            }
        };

        (@ $monster:ident resist Status :: $variant:ident) => {
            $monster.resistances.push($crate::processor::weapons::Status::$variant.into());
        };

        (@ $monster:ident resist Effect :: $variant:ident) => {
            $monster.resistances.push($crate::processor::monsters::effectives::EffectKind::$variant.into());
        };

        (@ $preset:ident => $monster:ident weak Status :: $variant:ident) => {
            $monster.weaknesses.push($crate::processor::monsters::effectives::Weakness::status(Status::$variant, $preset.tier.as_number()));
        };

        (@ $preset:ident => $monster:ident weak Effect :: $variant:ident) => {
            $monster.weaknesses.push($crate::processor::monsters::effectives::EffectKind::$variant.into());
        };
    }

    pub(super) use add_condition;
}
