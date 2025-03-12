use crate::is_weapon;
use crate::processor::weapons::heavy_bowgun::{Ammo, AmmoCapacityData, AmmoLevelData};
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
    special_ammo: SpecialAmmo,
    ammo: Vec<Ammo>,
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
}

is_weapon!(is_light_bowgun() => WeaponKindCode::LightBowgun);

impl From<&LightBowgunData> for LightBowgun {
    fn from(value: &LightBowgunData) -> Self {
        Self {
            ammo: Ammo::from_data(value.ammo_levels, value.ammo_capacities),
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
