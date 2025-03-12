use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::SwordShield,
        input_prefix: "ShortSword",
        output_prefix: Some("SwordShield"),
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SwordShield {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SwordShieldData {
    #[serde(rename = "_Type", deserialize_with = "is_sword_shield")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_sword_shield() => WeaponKindCode::SwordShield);

impl From<&SwordShieldData> for SwordShield {
    fn from(value: &SwordShieldData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
        }
    }
}
