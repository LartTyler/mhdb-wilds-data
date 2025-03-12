use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::DualBlades,
        input_prefix: "TwinSword",
        output_prefix: Some("DualBlades"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DualBlades {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct DualBladesData {
    #[serde(rename = "_Type", deserialize_with = "is_dual_blades")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_dual_blades() => WeaponKindCode::DualBlades);

impl From<&DualBladesData> for DualBlades {
    fn from(value: &DualBladesData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
        }
    }
}
