use crate::processor::monsters::id::Identifiers;
use crate::processor::monsters::parts::{PartKind, break_rewards};
use crate::processor::monsters::{Health, Monster};
use crate::processor::weapons::insect_glaive::KinsectEssenceKind;
use crate::processor::{FileObjects, Guid, Multiplier, Percentage, PercentageConversion, ReadFile};
use anyhow::Context;
use rslib::config::Config;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

pub const PARTS_DIR: &str = "user/monsters/parts";

pub(super) fn process(
    config: &Config,
    monsters: &mut FileObjects<Monster>,
    ids: &Identifiers,
) -> anyhow::Result<()> {
    let base_path = config.data_path(PARTS_DIR);

    for monster in monsters.items_mut() {
        log::trace!("Processing {}", monster.game_id);

        let path = base_path.join(monster.context.packs.monster_parts_file());
        let data = PartsData::read_file(path)?;

        monster.base_health = data.base_health;

        for data in data.parts {
            monster.parts.push(data.into());
        }

        monster.parts.sort_by_key(|v| v.kind);

        for data in data.multipliers {
            let multipliers = Multipliers::from(&data);

            for part in &mut monster.parts {
                if part.context.meat_guid == data.guid {
                    part.multipliers = multipliers.clone();
                }
            }
        }

        let linked_parts: HashMap<Guid, Guid> = data
            .linked_parts
            .into_iter()
            .filter_map(|mut v| v.targets.pop().map(|target| (v.guid, target)))
            .collect();

        for data in data.breakables {
            let part_guid = match data.kind {
                BreakTargetKind::Normal => &data.target,
                BreakTargetKind::Linked => linked_parts.get(&data.target).with_context(|| {
                    format!("Could not find linked GUID {:?} in lookup table", data.target)
                })?,
            };

            let part = monster.parts.iter_mut()
                .find(|v| v.context.guid == *part_guid)
                .with_context(|| {
                    format!("Could not find part with GUID {part_guid:?}")
                })?;

            part.context.break_guids.push(data.guid);
        }

        break_rewards::process(ids, &base_path, monster)?;
    }

    Ok(())
}

pub type BreakIndex = i16;

#[derive(Debug, Serialize)]
pub struct MonsterPart {
    pub kind: PartKind,
    base_health: Option<Health>,
    kinsect_essence: KinsectEssenceKind,
    pub multipliers: Multipliers,

    /// Contains context required to complete processing. These values should not be written to
    /// the final merged file.
    #[serde(skip)]
    pub context: MonsterPartContext,
}

impl From<PartData> for MonsterPart {
    fn from(value: PartData) -> Self {
        assert!(
            !value.has_health || !value.health.is_empty() && value.health[0] >= 0.0,
            "Part base health must exist and should be greater than zero.",
        );

        Self {
            kind: value.kind,
            base_health: value.has_health.then_some(value.health[0] as Health),
            kinsect_essence: value.kinsect_essence,
            multipliers: Multipliers::default(),
            context: MonsterPartContext {
                guid: value.guid,
                meat_guid: value.meat_guid,
                break_guids: Vec::new(),
                break_reward_indices: Vec::new(),
            },
        }
    }
}

#[derive(Debug)]
pub struct MonsterPartContext {
    guid: Guid,
    meat_guid: Guid,
    pub break_guids: Vec<Guid>,
    pub break_reward_indices: Vec<BreakIndex>,
}

#[derive(Debug, Deserialize)]
struct PartsData {
    #[serde(rename = "_BaseHealth")]
    base_health: Health,
    #[serde(rename = "_PartsArray")]
    parts: Vec<PartData>,
    #[serde(rename = "_MeatArray")]
    multipliers: Vec<MultiplierData>,
    #[serde(rename = "_PartsBreakArray")]
    breakables: Vec<BreakData>,
    #[serde(rename = "_MultiPartsArray")]
    linked_parts: Vec<LinkedPartsData>,
}

type PartHealth = f32;

#[derive(Debug, Deserialize)]
struct PartData {
    #[serde(rename = "_InstanceGuid")]
    guid: Guid,
    #[serde(rename = "_MeatGuidNormal")]
    meat_guid: Guid,
    #[serde(rename = "_PartsType")]
    kind: PartKind,
    #[serde(rename = "_Vital")]
    health: Vec<PartHealth>,
    #[serde(rename = "_RodExtract")]
    kinsect_essence: KinsectEssenceKind,
    #[serde(rename = "_IsEnablePartsVital")]
    has_health: bool,
}

#[derive(Debug, Serialize, Default, Clone)]
pub struct Multipliers {
    pub slash: Multiplier,
    pub blunt: Multiplier,
    pub pierce: Multiplier,
    pub fire: Multiplier,
    pub water: Multiplier,
    pub thunder: Multiplier,
    pub ice: Multiplier,
    pub dragon: Multiplier,
    pub stun: Multiplier,
}

impl From<&MultiplierData> for Multipliers {
    fn from(value: &MultiplierData) -> Self {
        Self {
            slash: value.slash.percent_as_f32(),
            blunt: value.blunt.percent_as_f32(),
            pierce: value.pierce.percent_as_f32(),
            fire: value.fire.percent_as_f32(),
            water: value.water.percent_as_f32(),
            thunder: value.thunder.percent_as_f32(),
            ice: value.ice.percent_as_f32(),
            dragon: value.dragon.percent_as_f32(),
            stun: value.stun.percent_as_f32(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct MultiplierData {
    #[serde(rename = "_InstanceGuid")]
    guid: Guid,
    #[serde(rename = "_Slash")]
    slash: Percentage,
    #[serde(rename = "_Blow")]
    blunt: Percentage,
    #[serde(rename = "_Shot")]
    pierce: Percentage,
    #[serde(rename = "_Fire")]
    fire: Percentage,
    #[serde(rename = "_Water")]
    water: Percentage,
    #[serde(rename = "_Thunder")]
    thunder: Percentage,
    #[serde(rename = "_Ice")]
    ice: Percentage,
    #[serde(rename = "_Dragon")]
    dragon: Percentage,
    #[serde(rename = "_Stun")]
    stun: Percentage,
}

#[derive(Debug, Deserialize)]
struct BreakData {
    #[serde(rename = "_InstanceGuid")]
    guid: Guid,
    #[serde(rename = "_TargetCategory")]
    kind: BreakTargetKind,
    #[serde(rename = "_TargetDataGuid")]
    target: Guid,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum BreakTargetKind {
    Normal = 0,
    Linked,
}

#[derive(Debug, Deserialize)]
struct LinkedPartsData {
    #[serde(rename = "_InstanceGuid")]
    guid: Guid,
    #[serde(rename = "_LinkPartsGuids")]
    targets: Vec<Guid>,
}
