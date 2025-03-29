use crate::processor::Processor;
use crate::should_run;
use rslib::config::Config;
use serde::Deserialize;

type StageId = isize;

pub(super) fn process(config: &Config, filters: &[Processor]) -> anyhow::Result<()> {
    should_run!(filters, Processor::Locations);

    Ok(())
}

#[derive(Debug, Deserialize)]
struct IdData {
    #[serde(rename = "_FixedID")]
    id: StageId,
    #[serde(rename = "_EnumName")]
    name: String,
    #[serde(rename = "_EnumValue")]
    value: u8,
}

impl IdData {
    fn bitmask(&self) -> u32 {
        1 << self.value
    }
}
