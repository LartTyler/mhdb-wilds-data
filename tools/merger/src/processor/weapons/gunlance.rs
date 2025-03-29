use crate::is_weapon;
use crate::processor::weapons::{
    HandicraftData, ProcessorDefinition, Sharpness, SharpnessData, WeaponKindCode,
};
use crate::processor::{values_until_first_zero, Processor};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub(super) fn definition() -> ProcessorDefinition {
    ProcessorDefinition {
        processor: Processor::Gunlance,
        input_prefix: "GunLance",
        output_prefix: Some("Gunlance"),
        callback: None,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct Gunlance {
    shell: ShellKind,
    shell_level: u8,
    sharpness: Sharpness,
    handicraft: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GunlanceData {
    #[serde(rename = "_Type", deserialize_with = "is_gunlance")]
    _type: WeaponKindCode,
    #[serde(rename = "_Wp07ShellType")]
    shell: ShellKind,
    #[serde(rename = "_Wp07ShellLv")]
    shell_level: ShellLevel,
    #[serde(rename = "_SharpnessValList")]
    sharpness: SharpnessData,
    #[serde(rename = "_TakumiValList")]
    handicraft: HandicraftData,
}

impl From<&GunlanceData> for Gunlance {
    fn from(value: &GunlanceData) -> Self {
        Self {
            shell: value.shell,
            shell_level: value.shell_level.as_level_number(),
            sharpness: Sharpness::from_data(value.sharpness),
            handicraft: values_until_first_zero(&value.handicraft),
        }
    }
}

is_weapon!(is_gunlance() => WeaponKindCode::Gunlance);

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all(serialize = "lowercase"))]
#[repr(isize)]
enum ShellKind {
    Normal = -324406336,
    Wide = -1732758016,
    Long = 203273856,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[repr(isize)]
enum ShellLevel {
    LV1 = 1226920576,
    LV2 = -993734528,
    LV3 = -745160128,
    LV4 = -170079472,
    LV5 = -269717152,
    LV6 = 145851744,
    LV7 = -58574980,
    LV8 = -1868644224,
}

impl ShellLevel {
    fn as_level_number(&self) -> u8 {
        match self {
            Self::LV1 => 1,
            Self::LV2 => 2,
            Self::LV3 => 3,
            Self::LV4 => 4,
            Self::LV5 => 5,
            Self::LV6 => 6,
            Self::LV7 => 7,
            Self::LV8 => 8,
        }
    }
}
