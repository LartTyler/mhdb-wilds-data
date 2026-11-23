use crate::processor::missions::{MissionId, SortValue};
use crate::processor::{ReadFile, Result};
use rslib::config::Config;
use serde::Deserialize;

const SORT_DATA: &str = "user/missions/FreeQuestSortData.json";

pub fn load(config: &Config) -> Result<Vec<SortData>> {
    let mut data: Vec<SortData> = Vec::read_file(config.data_path(SORT_DATA))?;
    data.sort_by_key(|v| !v.order);

    // For some weird reason, quests are sorted by their order property in DESCENDING order. Because
    // that's incredibly confusing, and also not consistent with how we're presenting sort data
    // for missions, we'll just reverse it ourselves.
    for (index, data) in data.iter_mut().enumerate() {
        data.order = index;
    }

    Ok(data)
}

#[derive(Debug, Deserialize)]
pub(super) struct SortData {
    #[serde(rename = "_MissionId")]
    pub mission_id: MissionId,
    #[serde(rename = "_Index")]
    pub order: SortValue,
}
