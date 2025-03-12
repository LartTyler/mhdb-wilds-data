use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{exclude_zeroes, Processor};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::SwitchAxe,
        input_prefix: "SlashAxe",
        output_prefix: Some("SwitchAxe"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SwitchAxe {
    sharpness: Sharpness,
    handicraft: Vec<u8>,
    phial: Phial,
}

#[derive(Debug, Deserialize)]
pub(super) struct SwitchAxeData {
    #[serde(rename = "_Type", deserialize_with = "is_switch_axe")]
    _type: WeaponKindCode,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
    #[serde(rename = "_Wp08BinType")]
    phial: PhialDataKind,
    #[serde(rename = "_Wp08BinValue")]
    phial_raw: u8,
}

is_weapon!(is_switch_axe() => WeaponKindCode::SwitchAxe);

impl From<&SwitchAxeData> for SwitchAxe {
    fn from(value: &SwitchAxeData) -> Self {
        Self {
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: exclude_zeroes(&value.handicraft),
            phial: Phial::from_data(value.phial, value.phial_raw),
        }
    }
}

#[derive(Debug, Deserialize_repr, Copy, Clone)]
#[repr(u8)]
enum PhialDataKind {
    Power = 0,
    Element,
    Dragon,
    Exhaust,
    Paralyze,
    Poison,
}

#[derive(Debug, Serialize)]
struct Phial {
    #[serde(flatten)]
    kind: PhialKind,
}

impl Phial {
    fn from_data(kind: PhialDataKind, raw: u8) -> Self {
        Self {
            kind: PhialKind::from_data(kind, raw),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "raw", rename_all = "lowercase")]
enum PhialKind {
    Power,
    Element,
    Dragon(u8),
    Exhaust(u8),
    Paralyze(u8),
    Poison(u8),
}

impl PhialKind {
    fn from_data(kind: PhialDataKind, raw: u8) -> Self {
        use PhialDataKind::*;

        match kind {
            Power => Self::Power,
            Element => Self::Element,
            Dragon => Self::Dragon(raw),
            Exhaust => Self::Exhaust(raw),
            Paralyze => Self::Paralyze(raw),
            Poison => Self::Poison(raw),
        }
    }
}
