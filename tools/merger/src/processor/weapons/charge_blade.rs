use crate::is_weapon;
use crate::processor::weapons::{ProcessorDefinition, WeaponKindCode};
use crate::processor::Processor;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::ChargeBlade,
        input_prefix: "ChargeAxe",
        output_prefix: Some("ChargeBlade"),
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ChargeBlade {
    phial: PhialKind,
}

impl From<&ChargeBladeData> for ChargeBlade {
    fn from(value: &ChargeBladeData) -> Self {
        Self { phial: value.phial }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct ChargeBladeData {
    #[serde(rename = "_Type", deserialize_with = "is_charge_blade")]
    _type: WeaponKindCode,
    #[serde(rename = "_Wp09BinType")]
    phial: PhialKind,
}

is_weapon!(is_charge_blade() => WeaponKindCode::ChargeBlade);

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(u8)]
enum PhialKind {
    Impact = 0,
    Element,
}
