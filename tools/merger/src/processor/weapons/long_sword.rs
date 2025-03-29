use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{values_until_first_zero, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::LongSword,
        input_prefix: "Tachi",
        output_prefix: Some("LongSword"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct LongSword {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LongSwordData {
    #[serde(rename = "_Type", deserialize_with = "is_long_sword")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_long_sword() => WeaponKindCode::LongSword);

impl From<&LongSwordData> for LongSword {
    fn from(value: &LongSwordData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: values_until_first_zero(&value.handicraft),
        }
    }
}
