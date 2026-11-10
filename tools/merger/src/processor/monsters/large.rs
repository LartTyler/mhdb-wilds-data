use crate::processor::LanguageMap;
use crate::processor::monsters::id::Identifiers;
use crate::processor::monsters::{Monster, MonsterKind, Variant};
use crate::serde::ordered_map;
use rslib::config::Config;
use serde::Serialize;

pub fn process(
    config: &Config,
    ids: &Identifiers,
    monster: &mut Monster,
) -> anyhow::Result<()> {
    Ok(())
}

#[derive(Debug, Serialize, Default)]
pub struct LargeMonster {
    pub variants: Vec<Variant>,
    #[serde(serialize_with = "ordered_map")]
    pub tips: LanguageMap,
}

impl From<LargeMonster> for MonsterKind {
    fn from(value: LargeMonster) -> Self {
        Self::Large(value)
    }
}
