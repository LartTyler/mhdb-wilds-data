use crate::is_weapon;
use crate::processor::weapons::{ProcessorDefinition, WeaponKindCode};
use crate::processor::Processor;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::HeavyBowgun,
        input_prefix: "HeavyBowgun",
        output_prefix: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct HeavyBowgun {
    ammo: Vec<Ammo>,
}

#[derive(Debug, Deserialize)]
pub(super) struct HeavyBowgunData {
    #[serde(rename = "_Type", deserialize_with = "is_heavy_bowgun")]
    _type: WeaponKindCode,
    #[serde(rename = "_ShellLv")]
    ammo_levels: AmmoLevelData,
    #[serde(rename = "_ShellNum")]
    ammo_capacities: AmmoCapacityData,
}

impl From<&HeavyBowgunData> for HeavyBowgun {
    fn from(value: &HeavyBowgunData) -> Self {
        Self {
            ammo: value
                .ammo_levels
                .iter()
                .zip(value.ammo_capacities)
                .enumerate()
                .filter_map(|(index, (level, capacity))| {
                    let level = level.to_level_number();

                    if level == 0 {
                        return None;
                    }

                    Some(Ammo {
                        kind: AmmoKind::from_index(index),
                        level,
                        capacity,
                    })
                })
                .collect(),
        }
    }
}

is_weapon!(is_heavy_bowgun() => WeaponKindCode::HeavyBowgun);

#[derive(Debug, Serialize, Hash)]
#[serde(rename_all = "lowercase")]
enum AmmoKind {
    Normal,
    Pierce,
    Spread,
    Slicing,
    Sticky,
    Cluster,
    Wyvern,
    Poison,
    Paralysis,
    Sleep,
    Flaming,
    Water,
    Freeze,
    Thunder,
    Dragon,
    Recover,
    Demon,
    Armor,
    Exhaust,
    Tranq,
}

impl AmmoKind {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Normal,
            1 => Self::Pierce,
            2 => Self::Spread,
            3 => Self::Sticky,
            4 => Self::Cluster,
            5 => Self::Slicing,
            6 => Self::Wyvern,
            7 => Self::Flaming,
            8 => Self::Water,
            9 => Self::Thunder,
            10 => Self::Freeze,
            11 => Self::Dragon,
            12 => Self::Poison,
            13 => Self::Paralysis,
            14 => Self::Sleep,
            15 => Self::Demon,
            16 => Self::Armor,
            17 => Self::Recover,
            18 => Self::Exhaust,
            19 => Self::Tranq,
            _ => panic!("Value out of range: {index}"),
        }
    }
}

#[derive(Debug, Serialize)]
struct Ammo {
    kind: AmmoKind,
    level: u8,
    capacity: u8,
}

type AmmoLevelData = [AmmoLevel; 20];
type AmmoCapacityData = [u8; 20];

#[derive(Debug, Deserialize_repr)]
#[repr(isize)]
enum AmmoLevel {
    None = -1067201536,
    LV1 = -29471984,
    LV2 = 1468794112,
    LV3 = 1769455744,
}

impl AmmoLevel {
    fn to_level_number(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::LV1 => 1,
            Self::LV2 => 2,
            Self::LV3 => 3,
        }
    }
}
