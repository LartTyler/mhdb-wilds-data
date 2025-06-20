use crate::placeholders::{ApplyContext, Placeholder};
use crate::processor::{
    to_ingame_rarity, IconColor, LanguageMap, PopulateStrings, Processor, ReadFile, Result,
    WriteFile,
};
use crate::serde::ordered_map;
use crate::should_run;
use indicatif::ProgressBar;
use rslib::config::Config;
use rslib::formats::msg::Msg;
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

pub type ItemId = isize;

const DATA: &str = "user/itemData.json";
const RECIPES: &str = "user/ItemRecipe.json";
const STRINGS: &str = "msg/Item.json";

const OUTPUT: &str = "merged/Item.json";

const IGNORED_IDS: &[ItemId] = &[
    1, 100, 280, 283, 284, 476, 690,
    // Special exclusions; see https://github.com/LartTyler/mhdb-wilds-data?tab=readme-ov-file#notes-2
    278, 409,
];

pub fn process(config: &Config, filters: &[Processor]) -> Result {
    should_run!(filters, Processor::Items);

    let data: Vec<ItemData> = Vec::read_file(config.io.output.join(DATA))?;
    let strings = Msg::read_file(config.io.output.join(STRINGS))?;
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
        Placeholder::process(&mut item.descriptions, &ApplyContext::empty());

        merged.push(item);
    }

    progress.finish_and_clear();

    let recipes: Vec<RecipeData> = Vec::read_file(config.io.output.join(RECIPES))?;
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
        item.recipes
            .sort_by_key(|v| v.inputs.iter().sum::<ItemId>());
    }

    progress.finish_and_clear();

    merged.sort_by_key(|v| v.game_id);
    merged.write_file(config.io.output.join(OUTPUT))
}

#[derive(Debug, Serialize)]
struct Item {
    game_id: ItemId,
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
    icon: IconKind,
    icon_id: u8,
    icon_color: IconColor,
    icon_color_id: u8,
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
            icon: value.icon,
            icon_id: value.icon as u8,
            icon_color: value.icon_color,
            icon_color_id: value.icon_color as u8,
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
    id: ItemId,
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
    #[serde(rename = "_IconType")]
    icon: IconKind,
    #[serde(rename = "_IconColor")]
    icon_color: IconColor,
}

#[derive(Debug, Deserialize)]
struct RecipeData {
    #[serde(rename = "_ResultItem")]
    output_id: ItemId,
    #[serde(rename = "_Num")]
    output_amount: u8,
    #[serde(rename = "_Item")]
    input_ids: Vec<ItemId>,
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

#[derive(Debug, Deserialize_repr, Serialize, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
enum IconKind {
    MysteryArtian = 0,
    MysteryMaterial = 1,
    Question = 2,
    Mushroom = 6,
    Egg = 7,
    Honey = 8,
    Plant = 9,
    Potion = 10,
    Powder = 11,
    Whetstone = 12,
    Pill = 13,
    Fish = 14,
    Meat = 15,
    Barrel = 16,
    Bomb = 17,
    TrapTool = 18,
    Trap = 19,
    Gem = 20,
    Smoke = 21,
    FishingRod = 22,
    Binoculars = 25,
    Knife = 26,
    Grill = 27,
    Voucher = 29,
    Certificate = 30,
    Coin = 31,
    Nut = 32,
    AmmoBasic = 33,
    Phial = 35,
    Web = 36,
    Seed = 37,
    Ore = 38,
    Bug = 39,
    Poop = 40,
    Medulla = 41,
    Bone = 42,
    Scale = 43,
    Hide = 44,
    Claw = 45,
    Shell = 46,
    Tail = 47,
    Wing = 48,
    Skull = 49,
    Plate = 50,
    Crystal = 52,
    ArmorSphere = 55,
    MysteryDecoration = 56,
    CampingKit = 61,
    SlingerAmmo = 62,
    CaptureNet = 63,
    AmmoSlug = 67,
    AmmoSpecial = 68,
    AmmoUtility = 69,
    AmmoHeavy = 70,
    Curative = 71,
    Drug = 72,
    Extract = 73,
    Mantle = 74,
    CookingCheese = 77,
    CookingMushroom = 78,
    CookingShellfish = 79,
    CookingGarlic = 80,
    CookingEgg = 81,
    Sprout = 87,
    #[serde(other)]
    Unknown = u8::MAX,
}
