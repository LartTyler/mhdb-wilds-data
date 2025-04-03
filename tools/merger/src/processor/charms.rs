use crate::processor::{LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};

const DATA: &str = "user/Charm.json";
const STRINGS: &str = "msg/Charm.json";

const OUTPUT: &str = "merged/Charm.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Charms);

    let data: Vec<CharmData> = Vec::read_file(config.io.output.join(DATA))?;
    let strings = Msg::read_file(config.io.output.join(STRINGS))?;

    let mut merged: Vec<Charm> = Vec::with_capacity(data.len());
    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let mut charm = Charm::from(&data);
        strings.populate(&data.name_guid, &mut charm.names);

        merged.push(charm);
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Charm {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

impl From<&CharmData> for Charm {
    fn from(value: &CharmData) -> Self {
        Self {
            game_id: value.id,
            names: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CharmData {
    #[serde(rename = "_Type")]
    id: isize,
    #[serde(rename = "_Name")]
    name_guid: String,
}
