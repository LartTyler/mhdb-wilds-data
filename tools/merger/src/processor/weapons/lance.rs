use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::Lance,
        input_prefix: "Lance",
        output_prefix: None,
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Lance {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct LanceData {
    #[serde(rename = "_Type", deserialize_with = "is_lance")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_lance() => WeaponKindCode::Lance);

impl From<&LanceData> for Lance {
    fn from(value: &LanceData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
        }
    }
}
