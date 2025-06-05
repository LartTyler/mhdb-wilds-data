use crate::placeholders::{ApplyContext, Placeholder};
use crate::processor::locations::StageId;
use crate::processor::monsters::large::effectives::{Resistance, SpecialKind, Weakness};
use crate::processor::monsters::large::identifiers::{IdentifierMap, Identifiers};
use crate::processor::monsters::large::parts::Part;
use crate::processor::monsters::large::rewards::Reward;
use crate::processor::monsters::large::size::Size;
use crate::processor::monsters::{
    CommonData, MonsterId, SpeciesKind, MONSTER_DATA, MONSTER_STRINGS, REFS_FIELD,
};
use crate::processor::{LanguageMap, Lookup, LookupMap, PopulateStrings, ReadFile, WriteFile};
use crate::serde::ordered_map;
use anyhow::Context;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::Serialize;
use strum::{EnumIter, IntoEnumIterator};

mod effectives;
mod identifiers;
mod locations;
mod parts;
mod rewards;
mod size;

const IGNORED_IDS: &[MonsterId] = &[-334290336];

const OUTPUT: &str = "merged/LargeMonsters.json";

#[derive(Debug, Default)]
struct RunContext {
    pub monsters: Vec<LargeMonster>,
    pub lookup: LookupMap,
    pub identifiers: Identifiers,
}

impl RunContext {
    pub fn new(identifiers: IdentifierMap) -> Self {
        Self {
            identifiers: Identifiers::new(identifiers),
            ..Default::default()
        }
    }

    pub fn find_monster_mut_or_panic(&mut self, game_id: MonsterId) -> &mut LargeMonster {
        self.lookup.find_or_panic_mut(game_id, &mut self.monsters)
    }

    pub fn find_monster_mut(&mut self, game_id: MonsterId) -> Option<&mut LargeMonster> {
        self.lookup.find_in_mut(game_id, &mut self.monsters)
    }
}

pub(super) fn process(config: &Config) -> anyhow::Result<()> {
    let field_refs = Msg::read_file(config.io.output.join(REFS_FIELD))?;
    let placeholders = ApplyContext::new(vec![&field_refs]);

    let data: Vec<CommonData> = Vec::read_file(config.io.output.join(MONSTER_DATA))?;
    let strings = Msg::read_file(config.io.output.join(MONSTER_STRINGS))?;

    let mut context = RunContext::new(identifiers::create_identifier_map(config)?);

    for data in data {
        if data.large_monster_icon == 0 || IGNORED_IDS.contains(&data.id) {
            continue;
        }

        let mut monster = LargeMonster::from(&data);
        strings.populate(&data.name_guid, &mut monster.names);

        // Some monsters are not implemented yet, which can be detected by the monster entry having
        // no names set in the translations file.
        if monster.names.is_empty() {
            continue;
        }

        strings.populate(&data.description_guid, &mut monster.descriptions);
        Placeholder::process(&mut monster.descriptions, &placeholders);

        strings.populate(&data.features_guid, &mut monster.features);
        Placeholder::process(&mut monster.features, &placeholders);

        strings.populate(&data.tips_guid, &mut monster.tips);
        Placeholder::process(&mut monster.tips, &placeholders);

        for variant in VariantKind::iter() {
            let mut names = LanguageMap::new();
            strings.populate(variant.get_guid(&data), &mut names);

            if !names.is_empty() {
                monster.variants.push(Variant {
                    kind: variant,
                    names,
                });
            }
        }

        let next_index = context.monsters.len();
        context.lookup.insert(monster.game_id, next_index);

        context.monsters.push(monster);
    }

    // Sequencing is important.
    size::process(config, &mut context)?;
    locations::process(config, &mut context)?;
    parts::process(config, &mut context)?;
    rewards::process(config, &mut context)?;

    // Must come after parts, as it depends on part damage multiplier data
    effectives::process(config, &mut context)?;

    let RunContext { mut monsters, .. } = context;

    monsters.sort_by_key(|v| v.game_id);

    monsters
        .write_file(config.io.output.join(OUTPUT))
        .context("Failed to write large monsters")
}

#[derive(Debug, Serialize)]
pub(super) struct LargeMonster {
    game_id: MonsterId,
    species: SpeciesKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    features: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    tips: LanguageMap,
    variants: Vec<Variant>,
    size: Size,
    base_health: u16,
    locations: Vec<StageId>,
    weaknesses: Vec<Weakness>,
    resistances: Vec<Resistance>,
    rewards: Vec<Reward>,
    parts: Vec<Part>,
}

impl LargeMonster {
    fn find_weakness_mut(&mut self, kind: SpecialKind) -> Option<&mut Weakness> {
        self.weaknesses.iter_mut().find(|v| v.kind == kind)
    }
}

impl From<&CommonData> for LargeMonster {
    fn from(value: &CommonData) -> Self {
        Self {
            game_id: value.id,
            species: value.species_kind,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            features: LanguageMap::new(),
            tips: LanguageMap::new(),
            variants: Vec::new(),
            size: Size::default(),
            base_health: 0,
            locations: Vec::new(),
            weaknesses: Vec::new(),
            resistances: Vec::new(),
            rewards: Vec::new(),
            parts: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Variant {
    kind: VariantKind,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
}

#[derive(Debug, Serialize, Ord, PartialOrd, Eq, PartialEq, EnumIter, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
enum VariantKind {
    Alpha,
    Tempered,
    Frenzied,
}

impl VariantKind {
    fn get_guid<'a>(&self, data: &'a CommonData) -> &'a str {
        match self {
            Self::Alpha => &data.alpha_name_guid,
            Self::Tempered => &data.tempered_name_guid,
            Self::Frenzied => &data.frenzied_name_guid,
        }
    }
}
