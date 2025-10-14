use crate::placeholders::{ApplyContext, Placeholder};
use crate::processor::{
    to_ingame_rarity, IdMap, LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DATA: &str = "user/AmuletData.json";
const STRINGS: &str = "msg/Amulet.json";
const RECIPES: &str = "user/AmuletRecipeData.json";

const OUTPUT: &str = "merged/Amulet.json";

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Amulets);

    let data: Vec<AmuletData> = Vec::read_file(config.io.output.join(DATA))?;
    let strings = Msg::read_file(config.io.output.join(STRINGS))?;

    let mut merged: Vec<Amulet> = Vec::with_capacity(data.len());
    let mut lookup: HashMap<isize, usize> = HashMap::new();

    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let existing_index = lookup.get(&data.group_id);
        let amulet = if let Some(index) = existing_index {
            &mut merged[*index]
        } else {
            let amulet = Amulet {
                game_id: data.group_id,
                is_random: data.is_random,
                ranks: Vec::new(),
            };

            let index = merged.len();
            lookup.insert(amulet.game_id, index);
            merged.push(amulet);

            &mut merged[index]
        };

        let mut rank = Rank::from(&data);

        strings.populate(&data.name_guid, &mut rank.names);

        strings.populate(&data.description_guid, &mut rank.descriptions);
        Placeholder::process(&mut rank.descriptions, &ApplyContext::empty());

        for (id, level) in data.skill_ids.into_iter().zip(data.skill_levels) {
            if id != 0 {
                rank.skills.insert(id, level);
            }
        }

        amulet.ranks.push(rank);
    }

    progress.finish_and_clear();

    let data: Vec<RecipeData> = Vec::read_file(config.io.output.join(RECIPES))?;
    let progress = ProgressBar::new(data.len() as u64);

    for data in data {
        progress.inc(1);

        let Some(amulet_index) = lookup.get(&data.amulet_id) else {
            continue;
        };
        let amulet = &mut merged[*amulet_index];
        let Some(rank) = amulet
            .ranks
            .iter_mut()
            .find(|v| v.level == data.amulet_level)
        else {
            continue;
        };

        rank.recipe.inputs = data
            .input_ids
            .into_iter()
            .zip(data.input_amounts)
            // Some crafting info contains inputs with zero quantity, which should be fine to
            // ignore. I'm guessing charms must always have a certain number of inputs in the game
            // files, even if they don't use them all.
            .filter(|(_id, amount)| *amount > 0)
            .collect();
    }

    progress.finish_and_clear();

    for amulet in &mut merged {
        amulet.ranks.sort_by_key(|v| v.level);
    }

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Amulet {
    game_id: isize,
    is_random: bool,
    ranks: Vec<Rank>,
}

#[derive(Debug, Serialize)]
struct Rank {
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    rarity: u8,
    level: u8,
    price: usize,
    #[serde(serialize_with = "ordered_map")]
    skills: IdMap,
    recipe: Recipe,
}

impl From<&AmuletData> for Rank {
    fn from(value: &AmuletData) -> Self {
        Self {
            rarity: to_ingame_rarity(value.rarity),
            level: value.level,
            price: value.price,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            skills: IdMap::new(),
            recipe: Recipe::default(),
        }
    }
}

#[derive(Debug, Serialize, Default)]
struct Recipe {
    #[serde(serialize_with = "ordered_map")]
    inputs: IdMap,
}
// 353607840 -1608406656 837371776 -1382563840
#[derive(Debug, Deserialize)]
struct AmuletData {
    #[serde(rename = "_AmuletType")]
    group_id: isize,
    #[serde(rename = "_Lv")]
    level: u8,
    #[serde(rename = "_Name")]
    name_guid: String,
    #[serde(rename = "_Explain")]
    description_guid: String,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_Price")]
    price: usize,
    #[serde(rename = "_Skill")]
    skill_ids: Vec<isize>,
    #[serde(rename = "_SkillLevel")]
    skill_levels: Vec<u8>,
    #[serde(rename = "_FreeFlag001")]
    is_random: bool,
}

#[derive(Debug, Deserialize)]
struct RecipeData {
    #[serde(rename = "_AmuletType")]
    amulet_id: isize,
    #[serde(rename = "_Lv")]
    amulet_level: u8,
    #[serde(rename = "_ItemId")]
    input_ids: Vec<isize>,
    #[serde(rename = "_ItemNum")]
    input_amounts: Vec<u8>,
}
