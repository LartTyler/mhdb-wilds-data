use crate::processor::{Processor, ReadFile, Result, WriteFile};
use crate::should_run;
use rslib::config::Config;
use serde::{Deserialize, Serialize};

const MATERIAL_DATA: &str = "user/facilities/foundry/SmallWorkshopItemData.json";
const ORE_DATA: &str = "user/facilities/foundry/SmallWorkshopDrillData.json";
const SPHERE_DATA: &str = "user/facilities/foundry/SmallWorkshopRefineData.json";

const MATERIAL_OUTPUT: &str = "merged/facilities/foundry/Materials.json";
const RESULTS_OUTPUT: &str = "merged/facilities/foundry/Results.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Foundry);

    let data: Vec<Material> = Vec::read_file(config.io.output.join(MATERIAL_DATA))?;
    data.write_file(config.io.output.join(MATERIAL_OUTPUT))?;

    let mut data: Vec<ItemOutput> = Vec::read_file(config.io.output.join(ORE_DATA))?;
    data.extend(Vec::read_file(config.io.output.join(SPHERE_DATA))?);

    data.write_file(config.io.output.join(RESULTS_OUTPUT))
}

#[derive(Debug, Deserialize, Serialize)]
struct Material {
    #[serde(alias = "_ItemId")]
    item_id: isize,

    #[serde(alias = "_RefinePoint")]
    armor_sphere_value: u16,

    #[serde(alias = "_DrillPoint")]
    ore_value: u16,
}

#[derive(Debug, Deserialize, Serialize)]
struct ItemOutput {
    #[serde(alias = "_ItemId")]
    item_id: isize,

    #[serde(alias = "_RefinePoint", alias = "_DrillPoint")]
    cost: u16,
}
