use crate::processor::{
    LanguageMap, Lookup, LookupMap, PopulateStrings, Processor, ReadFile, WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use rslib::formats::msg::{LanguageCode, Msg};
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::Deserialize_repr;
use std::collections::HashMap;

pub type StageId = isize;
pub type GimmickId = isize;

const STAGE_ID_DATA: &str = "data/Stage.json";
const GIMMICK_ID_DATA: &str = "data/GmID.json";

const DARK_AREA_DATA: &str = "data/DarkAreaSetting.json";
const GIMMICK_DATA: &str = "data/GimmickBasicData.json";
const GIMMICK_TEXT_DATA: &str = "data/GimmickTextData.json";

const CAMP_PATH_PREFIX: &str = "data/camps";

const STAGE_STRINGS: &str = "translations/RefEnvironment.json";
const GIMMICK_STRINGS: &str = "translations/Gimmick.json";

pub const OUTPUT: &str = "merged/Stage.json";

pub(super) fn process(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
    should_run!(filters, Processor::Locations);

    let data: Vec<StageIdData> = Vec::read_file(config.io.output.join(STAGE_ID_DATA))?;
    let strings = Msg::read_file(config.io.output.join(STAGE_STRINGS))?;

    let mut stages: Vec<Stage> = Vec::with_capacity(data.len());
    let mut lookup = LookupMap::with_capacity(data.len());

    for data in data {
        // Skip the first "INVALID" entry in the enum file.
        // Additionally, we only care about ST1XX stages, which are the actual field zones. I think.
        if data.value <= -1 || !data.name.starts_with("ST1") {
            continue;
        }

        let mut stage = Stage::from(&data);
        strings.populate(data.get_name_guid(), &mut stage.names);

        lookup.insert(stage.game_id, stages.len());
        stages.push(stage);
    }

    let data: Vec<DarkAreaData> = Vec::read_file(config.io.output.join(DARK_AREA_DATA))?;

    for data in data {
        let Some(stage) = lookup.find_in_mut(data.stage_id, &mut stages) else {
            continue;
        };

        // We determine the zone count by filtering the area numbers down to only the initial
        // sequential values. Some stages seem to include camps whose area is much higher than the
        // expected number of camps, e.g. ST101 (Windward Plains) includes areas 50 and 51, which
        // don't actually exist. My best guess is that those areas are for special story missions or
        // something like that.
        let area_count = data
            .areas
            .into_iter()
            .scan(1, |expected, v| {
                if v.number == *expected {
                    *expected = v.number + 1;
                    Some(v.number)
                } else {
                    None
                }
            })
            .max();

        stage.areas = area_count.expect("A field stage shouldn't have zero zones??");
    }

    let data: Vec<GimmickIdData> = Vec::read_file(config.io.output.join(GIMMICK_ID_DATA))?;
    let gimmick_ids: HashMap<_, _> = data.into_iter().map(|v| (v.id, v.name)).collect();

    let data: Vec<GimmickTextData> = Vec::read_file(config.io.output.join(GIMMICK_TEXT_DATA))?;
    let gimmick_text: HashMap<_, _> = data.into_iter().map(|v| (v.id, v)).collect();

    let data: Vec<GimmickData> = Vec::read_file(config.io.output.join(GIMMICK_DATA))?;
    let strings = Msg::read_file(config.io.output.join(GIMMICK_STRINGS))?;

    for data in data {
        if !data.is_tent() {
            continue;
        }

        let name = gimmick_ids
            .get(&data.id)
            .unwrap_or_else(|| panic!("Could not find gimmick with ID {}", data.id));

        let name = name.to_owned() + "_AaaUniqueParam.json";
        let path = config.io.output.join(CAMP_PATH_PREFIX).join(name);
        let camp_data = CampData::read_file(path)?;

        let stage = lookup.find_or_panic_mut(camp_data.stage_id, &mut stages);

        let mut camp = Camp::from(camp_data);
        camp.game_id = data.id;

        let text = gimmick_text
            .get(&data.id)
            .unwrap_or_else(|| panic!("Could not find gimmick with ID {}", data.id));

        strings.populate(&text.name_guid, &mut camp.names);

        // For some bizarre reason, some camps seem to have an area number that's outside the range
        // of areas in the stage. For example, "Crimson Rivulet" in the Oilwell Basin is in area
        // 12, but the game files say it's in area 27.
        //
        // Best guess (thank you Phy): the camps like this are in little offshoots on the map, which
        // could be different chunks of the map, separate from the actual zone itself. The game
        // sees it as a different area, but the label that the player sees matches what you'd expect
        // the area number to be.
        //
        // In such cases, the english name always follows the pattern "Area <num>: <name>". So, to
        // determine the real area number, all we need to do is parse `<num>` out of the name. It's
        // a bit hacky, but it's the best I think I can do right now.
        if camp.area > stage.areas {
            let en_name = strings
                .get_lang(&text.name_guid, LanguageCode::English)
                .unwrap_or_else(|| {
                    panic!(
                        "No english name found for fallback for gimmick ID {}",
                        data.id
                    )
                });

            let start = en_name.find(' ').unwrap_or_default() + 1;
            let end = en_name.find(':').unwrap_or(en_name.len());
            let fragment = &en_name[start..end];

            camp.area = fragment.parse()?;
        }

        stage.camps.push(camp);
    }

    for stage in &mut stages {
        stage.camps.sort_by_key(|v| v.area);
    }

    stages.sort_by_key(|v| v.game_id);
    stages.write_file(config.io.output.join(OUTPUT))?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub game_id: StageId,
    #[serde(serialize_with = "ordered_map")]
    pub names: LanguageMap,
    pub areas: u16,
    pub camps: Vec<Camp>,
    pub bitmask_value: u32,
}

impl From<&StageIdData> for Stage {
    fn from(value: &StageIdData) -> Self {
        Self {
            game_id: value.id,
            areas: 0,
            bitmask_value: value.bitmask(),
            names: LanguageMap::new(),
            camps: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct StageIdData {
    #[serde(rename = "_FixedID")]
    id: StageId,
    #[serde(rename = "_EnumName")]
    name: String,
    #[serde(rename = "_EnumValue")]
    value: i8,
}

impl StageIdData {
    fn bitmask(&self) -> u32 {
        1 << self.value
    }

    fn get_name_guid(&self) -> &str {
        // Mappings current as of 2025-03-31
        match self.name.as_ref() {
            "ST101" => "53c75773-e1c1-4842-b853-594c064c9dcf",
            "ST102" => "b05b96d2-3151-447c-911c-9e3d3b9e781c",
            "ST103" => "53dbc540-c48a-4c3d-bf1a-e7a715db927c",
            "ST104" => "c19b98a4-c220-4891-ac0e-15e21edf67bc",
            "ST105" => "2d17ecc9-6c48-4544-91ed-a078e05a4075",
            v => panic!("Unrecognized stage name {v}; you probably forgot to add a mapping :("),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Camp {
    game_id: GimmickId,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    area: u16,
    floor: u16,
    risk: Risk,
    position: Position,
}

impl From<CampData> for Camp {
    fn from(value: CampData) -> Self {
        Self {
            game_id: 0,
            names: LanguageMap::new(),
            area: value.area,
            floor: value.floor,
            risk: value.risk.into(),
            position: value.tent.position,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CampData {
    #[serde(rename = "_Stage", skip_serializing)]
    stage_id: StageId,
    #[serde(rename = "_AreaNum", deserialize_with = "negative_as_zero")]
    area: u16,
    #[serde(rename = "_FloorNum", deserialize_with = "negative_as_zero")]
    floor: u16,
    #[serde(rename = "_RiskDegree")]
    risk: RiskData,
    #[serde(rename = "_TentPoint")]
    tent: TentData,
}

#[derive(Debug, Deserialize)]
struct TentData {
    #[serde(rename = "_Position")]
    position: Position,
}

#[derive(Debug, Deserialize_repr)]
#[repr(u8)]
enum RiskData {
    Dangerous,
    Insecure,
    Safe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Risk {
    Dangerous,
    Insecure,
    Safe,
}

impl From<RiskData> for Risk {
    fn from(value: RiskData) -> Self {
        match value {
            RiskData::Dangerous => Self::Dangerous,
            RiskData::Insecure => Self::Insecure,
            RiskData::Safe => Self::Safe,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Deserialize)]
struct DarkAreaData {
    #[serde(rename = "_Stage")]
    stage_id: StageId,
    #[serde(rename = "_Area")]
    areas: Vec<AreaData>,
}

#[derive(Debug, Deserialize)]
struct AreaData {
    #[serde(rename = "_AreaNum")]
    number: u16,
}

fn negative_as_zero<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let v: i16 = Deserialize::deserialize(deserializer)?;
    Ok(u16::try_from(v).unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct GimmickData {
    #[serde(rename = "_GimmickId")]
    id: GimmickId,
    #[serde(rename = "_IconType")]
    icon_type: u16,
}

impl GimmickData {
    const ICON_TENT: u16 = 61;

    fn is_tent(&self) -> bool {
        self.icon_type == Self::ICON_TENT
    }
}

#[derive(Debug, Deserialize)]
struct GimmickTextData {
    #[serde(rename = "_GimmickId")]
    id: GimmickId,
    #[serde(rename = "_Name")]
    name_guid: String,
}

#[derive(Debug, Deserialize)]
struct GimmickIdData {
    #[serde(rename = "_FixedID")]
    id: GimmickId,
    #[serde(rename = "_EnumName")]
    name: String,
}
