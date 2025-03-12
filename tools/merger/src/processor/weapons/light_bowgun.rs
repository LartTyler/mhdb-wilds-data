use crate::is_weapon;
use crate::processor::weapons::heavy_bowgun::{self, AmmoCapacityData, AmmoKind, AmmoLevelData};
use crate::processor::weapons::{ProcessorDefinition, WeaponKindCode};
use crate::processor::Processor;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::LightBowgun,
        input_prefix: "LightBowgun",
        output_prefix: None,
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct LightBowgun {
    ammo: Vec<Ammo>,
    special_ammo: SpecialAmmo,
}

#[derive(Debug, Deserialize)]
pub(super) struct LightBowgunData {
    #[serde(rename = "_Type", deserialize_with = "is_light_bowgun")]
    _type: WeaponKindCode,
    #[serde(rename = "_Wp13SpecialAmmo")]
    special_ammo: SpecialAmmo,
    #[serde(rename = "_ShellLv")]
    ammo_levels: AmmoLevelData,
    #[serde(rename = "_ShellNum")]
    ammo_capacities: AmmoCapacityData,
    #[serde(rename = "_IsRappid")]
    ammo_rapid: AmmoRapidData,
}

is_weapon!(is_light_bowgun() => WeaponKindCode::LightBowgun);

impl From<&LightBowgunData> for LightBowgun {
    fn from(value: &LightBowgunData) -> Self {
        Self {
            ammo: Ammo::from_data(value.ammo_levels, value.ammo_capacities, value.ammo_rapid),
            special_ammo: value.special_ammo,
        }
    }
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(isize)]
enum SpecialAmmo {
    Wyvernblast = 1685175680,
    Adhesive = -1626714112,
}

#[derive(Debug, Serialize)]
struct Ammo {
    kind: AmmoKind,
    level: u8,
    capacity: u8,
    rapid: bool,
}

type AmmoRapidData = [bool; 20];

impl Ammo {
    fn from_data(
        levels: AmmoLevelData,
        capacities: AmmoCapacityData,
        rapid: AmmoRapidData,
    ) -> Vec<Self> {
        let mut ammo: Vec<_> = heavy_bowgun::Ammo::from_data(levels, capacities)
            .into_iter()
            .zip(rapid)
            .map(|(ammo, rapid)| {
                let mut ammo = Self::from(ammo);
                ammo.rapid = rapid;

                ammo
            })
            .collect();

        ammo.sort_by_key(|v| v.kind);
        ammo
    }
}

impl From<heavy_bowgun::Ammo> for Ammo {
    fn from(value: heavy_bowgun::Ammo) -> Self {
        Self {
            kind: value.kind,
            level: value.level,
            capacity: value.capacity,
            rapid: false,
        }
    }
}
