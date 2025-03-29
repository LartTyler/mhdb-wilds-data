use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{values_until_first_zero, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::GreatSword,
        input_prefix: "LongSword",
        output_prefix: Some("GreatSword"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct GreatSword {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GreatSwordData {
    #[serde(rename = "_Type", deserialize_with = "is_great_sword")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_great_sword() => WeaponKindCode::GreatSword);

impl From<&GreatSwordData> for GreatSword {
    fn from(value: &GreatSwordData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: values_until_first_zero(&value.handicraft),
        }
    }
}
