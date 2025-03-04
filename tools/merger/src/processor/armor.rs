use serde::Deserialize;
use crate::config::Config;
use crate::processor::Result;

pub fn process(config: &Config) -> Result {
    todo!()
}

#[derive(Debug, Deserialize)]
struct SeriesData {
    #[serde(rename = "_Series")]
    id: isize,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_Price")]
    price: usize,
}
