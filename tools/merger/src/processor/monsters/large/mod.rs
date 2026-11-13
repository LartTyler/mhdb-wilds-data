use crate::processor::monsters::large::size::MonsterSize;
use crate::processor::monsters::{Monster, MonsterKind, Variant};
use crate::processor::{FileObjects, LanguageMap};
use crate::serde::ordered_map;
use rslib::config::Config;
use serde::Serialize;

mod size;

pub fn process(config: &Config, monsters: &mut FileObjects<Monster>) -> anyhow::Result<()> {
    size::process(config, monsters)?;

    Ok(())
}

#[derive(Debug, Serialize, Default)]
pub struct LargeMonster {
    pub variants: Vec<Variant>,
    #[serde(serialize_with = "ordered_map")]
    pub tips: LanguageMap,
    pub size: MonsterSize,
}

impl From<LargeMonster> for MonsterKind {
    fn from(value: LargeMonster) -> Self {
        Self::Large(value)
    }
}
