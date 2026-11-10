use crate::processor::monsters::parts::PartKind;
use crate::processor::{Guid, LanguageMap, PopulateStrings, ReadFile, WriteFile};
use crate::serde::{optional_ordered_map, ordered_map};
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};

pub const DATA: &str = "user/monsters/EnemyPartsTypeData.json";
pub const STRINGS: &str = "msg/EnemyPartsTypeName.json";
pub const OUTPUT: &str = "PartNames.json";

pub fn process(config: &Config) -> anyhow::Result<()> {
    let data: Vec<PartTypeData> = Vec::read_file(config.data_path(DATA))?;
    let strings = Msg::read_file(config.data_path(STRINGS))?;

    let mut parts: Vec<Part> = Vec::with_capacity(data.len());

    for data in data {
        let mut part = Part {
            id: data.kind as PartId,
            kind: data.kind,
            name: LanguageMap::new(),
            description: LanguageMap::new(),
            rotten_name: None,
        };

        strings.populate(&data.name_guid, &mut part.name);
        strings.populate(&data.description_guid, &mut part.description);

        if !data.rotten_name_guid.is_empty() {
            let mut rotten_names = LanguageMap::new();
            strings.populate(&data.rotten_name_guid, &mut rotten_names);

            part.rotten_name = Some(rotten_names);
        }

        parts.push(part);
    }

    parts.sort_by_key(|v| v.id);
    parts.write_file(config.merged_path(OUTPUT))?;

    Ok(())
}

type PartId = isize;

#[derive(Debug, Serialize)]
pub struct Part {
    id: PartId,
    #[serde(flatten)]
    kind: PartKind,
    #[serde(serialize_with = "ordered_map")]
    name: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    description: LanguageMap,
    #[serde(serialize_with = "optional_ordered_map")]
    rotten_name: Option<LanguageMap>,
}

#[derive(Debug, Deserialize)]
pub struct PartTypeData {
    #[serde(rename = "_EmPartsType")]
    kind: PartKind,
    #[serde(rename = "_EmPartsName")]
    name_guid: Guid,
    #[serde(rename = "_EmRottenPartsName")]
    rotten_name_guid: Guid,
    #[serde(rename = "_EmPartsExp")]
    description_guid: Guid,
}
