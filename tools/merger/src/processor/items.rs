use crate::config::Config;
use crate::processor::{
    to_ingame_rarity, LanguageMap, PopulateStrings, Processor, ReadFile, Result, WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

const DATA: &str = "data/itemData.json";
const RECIPES: &str = "data/ItemRecipe.json";
const STRINGS: &str = "translations/Item.json";

const OUTPUT: &str = "merged/Item.json";

const IGNORED_IDS: &[isize] = &[
    1, 100, 280, 283, 284, 476, 690,
    // Special exclusions; see https://github.com/LartTyler/mhdb-wilds-data?tab=readme-ov-file#notes-2
    278, 409,
];

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Items);

    let data: Vec<ItemData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let strings = Msg::read_file(config.io.output_dir.join(STRINGS))?;
    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Item> = Vec::with_capacity(data.len());

    for data in data {
        progress.inc(1);

        if IGNORED_IDS.contains(&data.id) {
            continue;
        }

        let mut item = Item::from(&data);

        strings.populate(&data.name_guid, &mut item.names);
        strings.populate(&data.description_guid, &mut item.descriptions);

        merged.push(item);
    }

    progress.finish_and_clear();

    let recipes: Vec<RecipeData> = Vec::read_file(config.io.output_dir.join(RECIPES))?;
    let progress = ProgressBar::new(recipes.len() as u64);

    for recipe in recipes {
        progress.inc(1);

        let Some(item) = merged
            .iter_mut()
            .find(|item| item.game_id == recipe.output_id)
        else {
            continue;
        };

        item.recipes.push(recipe.into());
        item.recipes.sort_by_key(|v| v.inputs.iter().sum::<isize>());
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Item {
    game_id: isize,
    #[serde(serialize_with = "ordered_map")]
    names: LanguageMap,
    #[serde(serialize_with = "ordered_map")]
    descriptions: LanguageMap,
    kind: ItemKind,
    rarity: u8,
    max_count: u8,
    sell_price: usize,
    buy_price: usize,
    recipes: Vec<Recipe>,
    out_box: bool,
}

impl From<&ItemData> for Item {
    fn from(value: &ItemData) -> Self {
        Self {
            game_id: value.id,
            kind: value.kind,
            rarity: to_ingame_rarity(value.rarity),
            max_count: value.max_count,
            sell_price: value.sell_price,
            buy_price: value.buy_price,
            names: LanguageMap::new(),
            descriptions: LanguageMap::new(),
            recipes: Vec::new(),
            out_box: value.out_box,
        }
    }
}

#[derive(Debug, Serialize)]
struct Recipe {
    amount: u8,
    inputs: Vec<isize>,
}

impl From<RecipeData> for Recipe {
    fn from(value: RecipeData) -> Self {
        let mut inputs: Vec<_> = value
            .input_ids
            .into_iter()
            .filter(|v| !IGNORED_IDS.contains(v))
            .collect();

        inputs.sort();

        Self {
            amount: value.output_amount,
            inputs,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ItemData {
    #[serde(rename = "_ItemId")]
    id: isize,
    #[serde(rename = "_RawName")]
    name_guid: String,
    #[serde(rename = "_RawExplain")]
    description_guid: String,
    #[serde(rename = "_Type")]
    kind: ItemKind,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_MaxCount")]
    max_count: u8,
    #[serde(rename = "_SellPrice")]
    sell_price: usize,
    #[serde(rename = "_BuyPrice")]
    buy_price: usize,
    #[serde(rename = "_OutBox")]
    out_box: bool,
}

#[derive(Debug, Deserialize)]
struct RecipeData {
    #[serde(rename = "_ResultItem")]
    output_id: isize,
    #[serde(rename = "_Num")]
    output_amount: u8,
    #[serde(rename = "_Item")]
    input_ids: Vec<isize>,
}

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum ItemKind {
    Consumable = 0,
    Tool,
    Material,
    BowgunAmmo,
    BowCoating,
    Point,
    Mystery,
}
