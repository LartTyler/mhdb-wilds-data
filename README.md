# Tools
|Tool|Purpose|
|---|---|
|[REtool](https://residentevilmodding.boards.net/thread/10567/pak-tex-editing-tool)|Extracking data files from `.pak` (requires dtlnor's [MHWs.list](https://github.com/dtlnor/MonsterHunterWildsModding/blob/main/files/MHWs.list) file)|
|[REMSG_Converter](https://github.com/dtlnor/REMSG_Converter)|Convert `.msg.23` translation files into JSON|
|[RszTool](https://github.com/czastack/RszTool)|Convenient `.user.3` browsing and searching|

# Research
- Most files that we care about for the database project appear to be located in:
  - `natives/STM/GameDesign/Common/{Enemy, Equip, Item, Weapon}`
  - `natives/STM/GameDesign/Text/Excel_*`
- Every file I've examined so far appears to start with a dummy value as the first object. Maybe a template or base
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