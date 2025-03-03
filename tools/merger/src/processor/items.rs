use crate::config::Config;
use crate::processor::{LanguageMap, ReadFile, Result, Translations, WriteFile};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

const DATA: &str = "data/itemData.json";
const RECIPES: &str = "data/ItemRecipe.json";
const TRANSLATIONS: &str = "translations/Item.json";

const OUTPUT: &str = "merged/Item.json";

pub fn process(config: &Config) -> Result {
    let data: Vec<ItemData> = Vec::read_file(config.io.output_dir.join(DATA))?;
    let translations = Translations::read_file(config.io.output_dir.join(TRANSLATIONS))?;
    let progress = ProgressBar::new(data.len() as u64);

    let mut merged: Vec<Item> = Vec::with_capacity(data.len());

    for item in data {
        // ID 1 appears to be a placeholder item used in recipes with only one ingredient.
        if item.id == 1 {
            continue;
        }

        progress.inc(1);

        let mut names = LanguageMap::new();
        let mut descriptions = LanguageMap::new();

        for (index, lang) in translations.languages.iter().enumerate() {
            let name = translations.get_value(&item.name_guid, index);

            if let Some(name) = name {
                names.insert(*lang, name.to_owned());
            }

            let desc = translations.get_value(&item.description_guid, index);

            if let Some(desc) = desc {
                descriptions.insert(*lang, desc.to_owned());
            }
        }

        merged.push(Item {
            game_id: item.id,
            rarity: item.rarity,
            buy_price: item.buy_price,
            sell_price: item.sell_price,
            max_count: item.max_count,
            recipe: None,
            names,
            descriptions,
        });
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

        item.recipe = Some(Recipe {
            amount: recipe.output_amount,
            inputs: recipe
                .input_ids
                .iter()
                .cloned()
                .filter(|v| v != &1)
                .collect(),
        });
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output_dir.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Item {
    game_id: isize,
    names: LanguageMap,
    descriptions: LanguageMap,
    rarity: u8,
    max_count: u8,
    sell_price: usize,
    buy_price: usize,
    recipe: Option<Recipe>,
}

#[derive(Debug, Serialize)]
struct Recipe {
    amount: u8,
    inputs: Vec<isize>,
}

#[derive(Debug, Deserialize)]
struct ItemData {
    #[serde(rename = "_ItemId")]
    id: isize,
    #[serde(rename = "_RawName")]
    name_guid: String,
    #[serde(rename = "_RawExplain")]
    description_guid: String,
    #[serde(rename = "_Rare")]
    rarity: u8,
    #[serde(rename = "_MaxCount")]
    max_count: u8,
    #[serde(rename = "_SellPrice")]
    sell_price: usize,
    #[serde(rename = "_BuyPrice")]
    buy_price: usize,
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
