use crate::is_weapon;
use crate::processor::weapons::{ProcessorDefinition, WeaponKindCode};
use crate::processor::Processor;
use serde::{Deserialize, Serialize};

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::Bow,
        input_prefix: "Bow",
        output_prefix: None,
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Bow {
    coatings: Vec<Coating>,
}

impl From<&BowData> for Bow {
    fn from(value: &BowData) -> Self {
        Self {
            coatings: Coating::from_data(value.coatings),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct BowData {
    #[serde(rename = "_Type", deserialize_with = "is_bow")]
    _type: WeaponKindCode,
    #[serde(rename = "_isLoadingBin")]
    pub coatings: CoatingData,
}

is_weapon!(is_bow() => WeaponKindCode::Bow);

type CoatingData = [bool; 8];

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Coating {
    CloseRange,
    Power,
    Pierce,
    Paralysis,
    Poison,
    Sleep,
    Blast,
    Exhaust,
}

impl Coating {
    fn from_data(values: CoatingData) -> Vec<Self> {
        values
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| {
                value.then_some(match index {
                    0 => Self::CloseRange,
                    1 => Self::Power,
                    2 => Self::Pierce,
                    3 => Self::Paralysis,
                    4 => Self::Poison,
                    5 => Self::Sleep,
                    6 => Self::Blast,
                    7 => Self::Exhaust,
                    x => panic!("Unrecognized coating index {x}"),
                })
            })
            .collect()
    }
}
