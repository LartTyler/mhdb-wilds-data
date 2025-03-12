use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::Hammer,
        input_prefix: "Hammer",
        output_prefix: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Hammer {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct HammerData {
    #[serde(rename = "_Type", deserialize_with = "is_hammer")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

impl From<&HammerData> for Hammer {
    fn from(value: &HammerData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
        }
    }
}

is_weapon!(is_hammer() => WeaponKindCode::Hammer);
