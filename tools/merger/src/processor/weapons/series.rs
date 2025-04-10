use crate::processor::{LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

const DATA: &str = "user/weapons/WeaponSeriesData.json";
const ID_DATA: &str = "user/weapons/WeaponSeries.json";
const STRINGS: &str = "msg/WeaponSeries.json";

const OUTPUT: &str = "merged/WeaponSeries.json";

pub(super) type SeriesId = isize;

pub(super) fn process(config: &Config, filters: &[Processor]) -> Result<()> {
    should_run!(filters, Processor::WeaponSeries);

    let data: Vec<SeriesData> = Vec::read_file(config.io.output.join(DATA))?;
    let strings = Msg::read_file(config.io.output.join(STRINGS))?;

    let mut series: Vec<Series> = Vec::with_capacity(data.len());

    for data in data {
        let mut item = Series::from(&data);
        strings.populate(&data.name_guid, &mut item.names);

        series.push(item);
    }

    series.sort_by_key(|v| v.game_id);
    series.write_file(config.io.output.join(OUTPUT))?;

    Ok(())
}

pub(super) fn get_id_map(config: &Config, series_path: &Path) -> Result<HashMap<u8, SeriesId>> {
    let path = config.io.output.join(ID_DATA);
    let id_lookup: Vec<SeriesIdData> = Vec::read_file(path)?;
    let id_lookup: HashMap<u16, SeriesId> =
        id_lookup.into_iter().map(|v| (v.value, v.fixed)).collect();

    let path = config.io.output.join(series_path);
    let row_lookup: Vec<SeriesRowData> = Vec::read_file(path)?;

    Ok(row_lookup
        .into_iter()
        .map(|v| {
            let Some(series_id) = id_lookup.get(&v.simple_id) else {
                panic!("Could not find series ID from index {}", v.simple_id);
            };

            (v.row, *series_id)
        })
        .collect())
}

#[derive(Debug, Serialize)]
struct Series {
    game_id: SeriesId,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&SeriesData> for Series {
    fn from(value: &SeriesData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SeriesIdData {
    #[serde(rename = "_FixedID")]
    fixed: SeriesId,
    #[serde(rename = "_EnumValue")]
    value: u16,
}

#[derive(Debug, Deserialize)]
struct SeriesData {
    #[serde(rename = "_Series")]
    id: SeriesId,
    #[serde(rename = "_Name")]
    name_guid: String,
}

#[derive(Debug, Deserialize)]
struct SeriesRowData {
    #[serde(rename = "_Series")]
    simple_id: u16,
    #[serde(rename = "_RowLevel")]
    row: u8,
}
