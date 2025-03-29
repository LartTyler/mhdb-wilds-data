use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{values_until_first_zero, Processor};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::ChargeBlade,
        input_prefix: "ChargeAxe",
        output_prefix: Some("ChargeBlade"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ChargeBlade {
    phial: PhialKind,
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

impl From<&ChargeBladeData> for ChargeBlade {
    fn from(value: &ChargeBladeData) -> Self {
        Self {
            phial: value.phial,
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: values_until_first_zero(&value.handicraft),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ChargeBladeData {
    #[serde(rename = "_Type", deserialize_with = "is_charge_blade")]
    _type: WeaponKindCode,
    #[serde(rename = "_Wp09BinType")]
    phial: PhialKind,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

is_weapon!(is_charge_blade() => WeaponKindCode::ChargeBlade);

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(u8)]
enum PhialKind {
    Impact = 0,
    Element,
}
