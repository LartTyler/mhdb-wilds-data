use crate::placeholders::Placeholders;
use crate::processor::monsters::id::Identifiers;
use crate::processor::monsters::large::LargeMonster;
use crate::processor::monsters::species::SpeciesKind;
use crate::processor::monsters::{Monster, MonsterContext, MonsterId, MonsterKind, SmallMonster, Variant, VariantKind};
use crate::processor::{FileObjects, Guid, LanguageMap, PopulateStrings, ReadFile};
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::Deserialize;
use strum::IntoEnumIterator;

const STRINGS: &str = "msg/EnemyText.json";
const DATA: &str = "user/monsters/EnemyData.json";

pub fn load(config: &Config, ids: &Identifiers, placeholders: &Placeholders) -> anyhow::Result<FileObjects<Monster>> {
    let strings = Msg::read_file(config.data_path(STRINGS))?;

    let data: Vec<CommonData> = Vec::read_file(config.data_path(DATA))?;
    let mut monsters: FileObjects<Monster> = FileObjects::with_capacity(data.len());

    for data in data.into_iter().filter(|v| v.species != SpeciesKind::None) {
        let kind: MonsterKind = if data.is_large() {
            let mut monster = LargeMonster::default();

            strings.populate(&data.tips, &mut monster.tips);
            placeholders.apply(&mut monster.tips);

            for variant in VariantKind::iter() {
                let mut name = LanguageMap::default();
                strings.populate(variant.get_guid(&data), &mut name);

                if !name.is_empty() {
                    monster.variants.push(Variant {
                        kind: variant,
                        name,
                    });
                }
            }

            monster.into()
        } else {
            SmallMonster.into()
        };

        let mut monster = Monster {
            game_id: data.monster_id,
            kind,
            species: data.species,
            name: LanguageMap::new(),
            description: LanguageMap::new(),
            features: LanguageMap::new(),
            weaknesses: Vec::new(),
            resistances: Vec::new(),
            base_health: 0,
            parts: Vec::new(),
            rewards: Vec::new(),
            locations: Vec::new(),
            context: MonsterContext {
                packs: ids.load_file_packs(config, data.monster_id)?,
            },
        };

        strings.populate(&data.names.normal, &mut monster.name);

        strings.populate(&data.description, &mut monster.description);
        placeholders.apply(&mut monster.description);

        strings.populate(&data.features, &mut monster.features);
        placeholders.apply(&mut monster.features);

        monsters.add(monster);
    }

    Ok(monsters)
}

#[derive(Debug, Deserialize)]
pub(super) struct CommonData {
    #[serde(rename = "_enemyId")]
    pub monster_id: MonsterId,
    #[serde(rename = "_Species")]
    pub species: SpeciesKind,
    #[serde(flatten)]
    pub names: NameGuids,
    #[serde(rename = "_EnemyExp")]
    pub description: Guid,
    #[serde(rename = "_EnemyFeatures")]
    pub features: Guid,
    #[serde(rename = "_EnemyTips")]
    pub tips: Guid,
    #[serde(rename = "_BossIconType")]
    large_icon: u8,
}

impl CommonData {
    pub fn is_large(&self) -> bool {
        self.large_icon != 0
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct NameGuids {
    #[serde(rename = "_EnemyName")]
    pub normal: Guid,
    #[serde(rename = "_EnemyExtraName")]
    pub alpha: Guid,
    #[serde(rename = "_EnemyFrenzyName")]
    pub frenzied: Guid,
    #[serde(rename = "_EnemyLegendaryName")]
    pub tempered: Guid,
    #[serde(rename = "_EnemyLegendaryKingName")]
    pub arch_tempered: Guid,
}
