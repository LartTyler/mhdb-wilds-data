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
  - [Items](#items)
    - [Data Files](#data-files-2)
    - [Translation Files](#translation-files-2)
    - [Notes](#notes-2)


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

Unlike the Attack Boost sections, `_value` starts with what looks like the +5 attack modifier, instead of the percent
modifier argument.

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