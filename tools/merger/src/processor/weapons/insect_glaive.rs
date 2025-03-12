use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::InsectGlaive,
        input_prefix: "Rod",
        output_prefix: Some("InsectGlaive"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct InsectGlaive {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
    kinsect_level: u8,
}

#[derive(Debug, Deserialize)]
pub(super) struct InsectGlaiveData {
    #[serde(rename = "_Type", deserialize_with = "is_insect_glaive")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
    #[serde(rename = "_RodInsectLv")]
    kinsect_level: KinsectLevel,
}

is_weapon!(is_insect_glaive() => WeaponKindCode::InsectGlaive);

impl From<&InsectGlaiveData> for InsectGlaive {
    fn from(value: &InsectGlaiveData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
            kinsect_level: value.kinsect_level.to_level_number(),
        }
    }
}

#[derive(Debug, Deserialize_repr)]
#[repr(isize)]
enum KinsectLevel {
    LV1 = 1458810624,
    LV2 = -2011731328,
    LV3 = 1657462528,
    LV4 = 2092273536,
    LV5 = -48500096,
    LV6 = 456816000,
    LV7 = -225899808,
    LV8 = 1416307328,
    LV9 = 318816128,
    LV10 = -1267020544,
}

impl KinsectLevel {
    fn to_level_number(&self) -> u8 {
        match self {
            Self::LV1 => 1,
            Self::LV2 => 2,
            Self::LV3 => 3,
            Self::LV4 => 4,
            Self::LV5 => 5,
            Self::LV6 => 6,
            Self::LV7 => 7,
            Self::LV8 => 8,
            Self::LV9 => 9,
            Self::LV10 => 10,
        }
    }
}
