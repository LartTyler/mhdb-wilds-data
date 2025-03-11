use crate::config::Config;
use crate::processor::weapons::{AttributeKind, Crafting, CraftingTreeData, Special};
use crate::processor::{
    to_ingame_rarity, LanguageMap, Lookup, LookupMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::{should_run, weapon_data_struct, weapon_recipe_data, weapon_struct};
use rslib::formats::msg::Msg;
use serde::Serialize;
use std::collections::HashMap;

const DATA: &str = "data/Bow.json";
const RECIPES: &str = "data/BowRecipe.json";
const TREE: &str = "data/BowTree.json";
const STRINGS: &str = "translations/Bow.json";

const OUTPUT: &str = "merged/weapons/Bow.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Bow);

    let data: Vec<BowData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let strings = Msg::read_file(config.io.output_dir.join(STRINGS))?;

    let mut merged = Vec::with_capacity(data.len());
    let mut lookup: LookupMap<u32> = LookupMap::with_capacity(data.len());

    for data in data {
        let mut bow = Bow::from(&data);

        strings.populate(&data.name_guid, &mut bow.names);
        strings.populate(&data.description_guid, &mut bow.descriptions);

        if data.attribute != AttributeKind::None {
            bow.specials.push(Special {
                kind: data.attribute.into(),
                raw_damage: data.attribute_value_raw,
                hidden: false,
            });
        }

        if data.hidden_attribute != AttributeKind::None {
            bow.specials.push(Special {
                kind: data.hidden_attribute.into(),
                raw_damage: data.hidden_attribute_value_raw,
                hidden: true,
            });
        }

        lookup.insert(data.id, merged.len());
        merged.push(bow);
    }

    let data: Vec<Recipe> = Vec::read_file(config.io.output_dir.join(RECIPES))?;

    for data in data {
        let bow = lookup
            .find_in_mut(data.weapon_id, &mut merged)
            .unwrap_or_else(|| panic!("Could not find bow by ID: {}", data.weapon_id));

        bow.crafting.is_shortcut = data.is_shortcut;
        bow.crafting.inputs = data
            .item_ids
            .into_iter()
            .zip(data.item_amounts)
            .filter(|(id, _)| *id != 0)
            .collect();
    }

    let data: Vec<CraftingTreeData> = Vec::read_file(config.io.output_dir.join(TREE))?;
    let tree_guids: HashMap<String, u32> =
        data.iter().map(|v| (v.guid.clone(), v.weapon_id)).collect();

    for data in data {
        let bow = lookup
            .find_in_mut(data.weapon_id, &mut merged)
            .unwrap_or_else(|| panic!("Could not find bow by ID: {}", data.weapon_id));

        bow.crafting.column = data.column;
        bow.crafting.row = data.row;

        if !data.previous_guid.is_empty() {
            bow.crafting.previous_id = Some(*tree_guids.get(&data.previous_guid[0]).unwrap());
        }

        for guid in data.branch_guids {
            let branch_id = tree_guids.get(&guid).cloned();
            bow.crafting.branches.push(branch_id.unwrap());
        }

        bow.crafting.branches.sort();
    }

    merged.write_file(config.io.output_dir.join(OUTPUT))
}

weapon_data_struct! {
    struct BowData {
        #[serde(rename = "_Bow")]
        id: u32,
        #[serde(rename = "_isLoadingBin")]
        coatings: [bool; 8],
    }
}

weapon_recipe_data! {
    struct Recipe {
        #[serde(rename = "_Bow")]
        weapon_id: u32,
    }
}

weapon_struct! {
    struct Bow {
        coatings: Vec<Coating>,
    }
}

impl From<&BowData> for Bow {
    fn from(value: &BowData) -> Self {
        Self {
            game_id: value.id,
            kind: value.kind,
            rarity: to_ingame_rarity(value.rarity),
            attack_raw: value.attack_raw,
            defense: value.defense,
            affinity: value.critical,
            crafting: Crafting {
                zenny_cost: value.price,
                ..Default::default()
            },
            slots: value.slots.into_iter().filter(|v| *v > 0).collect(),
            coatings: Coating::from_data(value.coatings),
            skills: value
                .skill_ids
                .into_iter()
                .zip(value.skill_levels)
                .filter(|(id, _level)| *id != 0)
                .collect(),
            specials: Vec::new(),
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Coating {
    CloseRange,
    Power,
    Pierce,
    Paralysis,
    Poison,
    Sleep,
    Blast,
    Exhaust,
}

impl Coating {
    fn from_data(values: [bool; 8]) -> Vec<Self> {
        values
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| {
                if value {
                    let coating = match index {
                        0 => Self::CloseRange,
                        1 => Self::Power,
                        2 => Self::Pierce,
                        3 => Self::Paralysis,
                        4 => Self::Poison,
                        5 => Self::Sleep,
                        6 => Self::Blast,
                        7 => Self::Exhaust,
                        x => panic!("Unrecognized coating index {x}"),
                    };

                    Some(coating)
                } else {
                    None
                }
            })
            .collect()
    }
}
