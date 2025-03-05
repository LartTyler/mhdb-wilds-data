- [About](#about)
  - [Requirements](#requirements)
  - [Pipeline](#pipeline)
- [Tools](#tools)
- [Research](#research)
  - [Decorations (Accessories)](#decorations-accessories)
    - [Data Files](#data-files)
    - [Translation Files](#translation-files)
    - [Notes](#notes)
  - [Skills](#skills)
    - [Data Files](#data-files-1)
    - [Translation Files](#translation-files-1)
    - [Notes](#notes-1)
    - [Modifier Values?](#modifier-values)
    - [Skill Category](#skill-category)
  - [Items](#items)
    - [Data Files](#data-files-2)
    - [Translation Files](#translation-files-2)
    - [Notes](#notes-2)
  - [Armor](#armor)
    - [Data Files](#data-files-3)
    - [Translation Files](#translation-files-3)
    - [Notes](#notes-3)

# About
The goal of this project is to "glue" several other tools together in order to get sane JSON files for data objects in Wilds.

## Requirements
- C# .NET 8.0
- Rust

The C# project in `/tools/DotUserReader` (for now) needs to be manually compiled before running the project. Everything
else either has an executable embedded in the project, or will compile on-demand if necessary.

## Pipeline
1. Extract the `re_chunk_000.pak` file in the root of your Wilds install using `/tools/REtool/Extract-PAK.bat`.
2. Copy `/tools/extractor/examples/config.toml` to the project root, and adjust the paths in `[io]` to point to the
   extracted files. If you used the `bat` file as-is, no changes will be necessary.
3. Run `/extract.bat` to convert the relevant `.user.3` and `.msg.23` files to JSON dumps.
4. Run `/merge.bat` to convert the raw JSON dumps into a merged JSON format.

# Tools
|Tool|Purpose|
|---|---|
|[REtool](https://residentevilmodding.boards.net/thread/10567/pak-tex-editing-tool)|Extracting data files from `.pak` (requires dtlnor's [MHWs.list](https://github.com/dtlnor/MonsterHunterWildsModding/blob/main/files/MHWs.list) file)|
|[REMSG_Converter](https://github.com/dtlnor/REMSG_Converter)|Convert `.msg.23` translation files into JSON|
|[RszTool](https://github.com/czastack/RszTool)|Convenient `.user.3` browsing and searching|

# Research
Most files that we care about for the database project appear to be located in:
- `natives/STM/GameDesign/Common/{Enemy, Equip, Item, Weapon}`
- `natives/STM/GameDesign/Text/Excel_*`

Every file I've examined so far appears to start with a dummy value as the first object. Maybe a template or base
object? Something like that? Regardless, I'm like 99% certain we can ignore the first definition in every `.user.3`
file.

The rarity field (usually `_Rare`) in each file makes _absolultely no sense_. The value in the files seems to be
counting _down_ from 18, with an in-game rarity value of "1" corresponding to an in-file value of "18". I feel like I
must be missing something here, but for now I'm just going to "convert" it to the in-game value by subtracting the
in-file value from 19. This feels so hacky, and like it's going to bite me in the ass at some point.

## Decorations (Accessories)
### Data Files
- `natives/STM/GameDesign/Common/Equip/AccessoryData.user.3`

There are two other files that may be of note:
- `natives/STM/GameDesign/Common/Equip/AccessoryJudgeData.user.3`
- `natives/STM/GameDesign/Common/Equip/AccessoryRankJudgeData.user.3`

Those two files appear to contain drop chances, but I'm not 100% certain. Since they aren't relevant to any field
already in the database, I'm going to ignore them for now.

### Translation Files
- `natives/STM/GameDesign/Text/Excel_Equip/Accessory.msg.23`

There is a second file in the same directory named `AccessoryData.msg.23` that doesn't actually appear to contain any
information. All the translations are showing as empty, and there's only 4 entries. Ignoring for now.

### Notes
This one seems to be pretty straightforward. The data file contains all the decorations, and the translation file
contains all the relevant strings. The `_Skill` array in the data file contains the skill IDs the decoration grants,
and the `_SkillLevel` array contains each skill's level at the matching index.

The one field I'm not sure about is `_AccessoryType`. I have no idea what that value corresponds to, or if we even care
about it for the purpose of the database project.

## Skills
### Data Files
- `natives/STM/GameDesign/Common/Equip/SkillCommonData.user.3`
- `natives/STM/GameDesign/Common/Equip/SkillData.user.3`

### Translation Files
- `natives/STM/GameDesign/Text/Excel_Equip/SkillCommon.msg.23`
- `natives/STM/GameDesign/Text/Excel_Equip/Skill.msg.23`

### Notes
`SkillCommonData.user.3` appears to contain the actual skill data, things like category, and name and description GUIDs.

`SkillData.user.3` contains the information for each _level_ of a skill. There's both a `skillName` and `skillExplain`
GUID present in the file, but only `skillExplain` seems to hold a GUID that's actually useful. The GUID for the name
field seems to uniformly point to unique entries, but each one containing only placeholder / blank data. Each entry in
this file has a `skillId` field which points to the `skillId` field in `SkillCommonData.user.3`.

Basically, in order to get all the information on a skill and it's levels, you'll need both files. IMO the best option
would be to parse `SkillCommonData.user.3` first, index each entry by its `skillId` field, then parse `SkillData.user.3`
to get the levels and add them to your partial skills last.

Similar to [Items](#items), it looks like there's two skills whose IDs are `0` and `1` that contain no real information.
I believe both can ignored can be ignored.

### Modifier Values?
In `SkillData.user.3`, there's a `_value` field that appears to hold some sort of attribute modifiers for the skill. For
example, the entry for the "Attack Boost" skill at level 1 is:

```json
{
    "_Index": 1,
    "_dataId": 2,
    "_skillId": 1,
    "_SkillLv": 1,
    "_skillName": "824fa7e2-6344-4f5d-a140-0d411ccc674d",
    "_skillExplain": "96e65c81-c4a1-4fb0-aea4-c256200cda88",
    "_openSkill": [
        1,
        0,
        0,
        0,
        0
    ],
    "_value": [
        100,
        3,
        0,
        0
    ]
}
```

In-game, attack boost shows a +3 to attack (`_value[1]`). Later levels of attack boost _also_ give a percentage
increase to attack, and it seems that even for ranks that do not give that bonus, they still require the modifier to be
present (thus the "100" modifier at `_value[0]`). At later ranks that _do_ include the percent bonus, that value has
changed:

```json
{
    "_Index": 5,
    "_dataId": 6,
    "_skillId": 1,
    "_SkillLv": 5,
    "_skillName": "ee33bf9d-18fa-45cf-a868-25ce715a57a5",
    "_skillExplain": "6df1df71-77f5-4bff-bdda-aa73a34ef034",
    "_openSkill": [
        1,
        0,
        0,
        0,
        0
    ],
    "_value": [
        104,
        9,
        0,
        0
    ]
}
```

Those values match Attack Boost 5's +9 attack and +4% attack bonuses. I'm wondering if maybe those values are provided
to a constructor or initialization function, and maybe perhaps change their meaning based on the skill? To support this,
"Resentment" gives +5 attack when you have recoverable health. The JSON below is the relevant section from
`SkillData.user.3`.

```json
{
    "_Index": 8,
    "_dataId": 181,
    "_skillId": 1359821952,
    "_SkillLv": 1,
    "_skillName": "cb317928-7cd0-4a62-a063-1615e80dfa4c",
    "_skillExplain": "1b599bee-8dd2-45f0-9ad8-e1eac54f00b5",
    "_openSkill": [
        1359821952,
        0,
        0,
        0,
        0
    ],
    "_value": [
        5,
        0,
        0,
        0
    ]
}
```

### Skill Category
The `_skillCategory` field might actually be relevant to the API. It looks like it can be a value between 0 and
3 (inclusive), and so far I've found a distinction between set bonuses and actual concrete skills. For example,
the low rank Rey Dau set has a set bonus named "Rey Dau's Voltage", but the entries in `SkillData.user.3` point to name
and description entries in `Skill.msg.23` with actual values, not the dummy values found in most skill rank entries.
Additionally, the `_Lv` field of such entries appear to hold the number of set pieces required to activate the bonus.

Unlike the Attack Boost sections, `_value` starts with what looks like the +5 attack modifier, instead of the percent
modifier argument. Below is a table of what I believe those category values represent.

|Value|Description|Example|
|---|---|---|
|0|"Normal" skills (though maybe armor-only skills?)|Constitution, Speed Eating|
|1|Armor set bonuses|Thunderous Roar I (Rey Dau's Voltage)|
|2|Group skills, set bonuses granted by armor pieces belonging to the same category (such as guardian armor)|Fortifying Pelt, Guardian's Pulse|
|3|Weapon-only skills|Attack Boost, Critical Draw|

## Items
### Data Files
- `natives/STM/GameDesign/Common/ItemData.user.3`
- `natives/STM/GameDesign/Common/ItemRecipe.user.3`

### Translation Files
- `natives/STM/GameDesign/Text/Excel_Data/Item.msg.23`

### Notes
For recipe data, it looks like every item with a recipe _always_ has two IDs listed as an input. For items with only one
input, it seems like one of those IDs is always `1`, which points to an item in the files with no name or other
information. My guess is that this is just an empty item and is ignored by the game when displaying or crafting recipes.

Fields listed below are my best guess, based on which items have the flag set.

|Field|Type|Description|Example|
|---|---|---|---|
|`_Infinit`|boolean|Item isn't consumed on use|Capture Net|
|`_ForMoney`|boolean|Item is a treasure item|Silver Egg|
|`_Battle`|boolean|Item is a trap or slinger ammo|Screamer pod, drugged meat, shock trap|
|`_Shikyu`|boolean|Supply items|First-aid med|

## Armor
### Data Files
- `natives/STM/GameDesign/Equip/ArmorData.user.3`
- `natives/STM/GameDesign/Equip/ArmorRecipeData.user.3`
- `natives/STM/GameDesign/Equip/ArmorSeriesData.user.3`

### Translation Files
- `natives/STM/GameDesign/Text/Excel_Equip/Armor.msg.23`
- `natives/STM/GameDesign/Text/Excel_Equip/ArmorSeries.msg.23`

### Notes
In `ArmorData.user.3`, the `_Skill` array appears to give special meaning to the position of a skill in the array. The
skill at index 0 seems to always be the set bonus, or `0` if there isn't one, followed by the group bonus at index 1
(again, or `0` if there isn't one), followed by general skills in the remainder of the array.

In `ArmorSeriesData.user.3`, there are two "special" IDs that I've found:
- `0`, which seems to correspond to an empty / "template" value that several other objects tend to have.
- `1`, which seems to be for when no armor is equipped.

ID `0` does not appear in `ArmorData.user.3`, but ID `1` does. Neither are relevant to the API, so they will not be
included in merged files.