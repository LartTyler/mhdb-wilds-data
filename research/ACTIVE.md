- [Missions](#missions)
  - [Entrypoints](#entrypoints)
    - [MissionActivityData.user.3](#missionactivitydatauser3)
- [Missions (old)](#missions-old)
  - [Entrypoints](#entrypoints-1)
    - [MissionActivityData.user.3](#missionactivitydatauser3-1)
    - [QuestOpenData.user.3](#questopendatauser3)
    - [Walking `Mission/Mission*` directories](#walking-missionmission-directories)

# Missions
I have found two different mechanisms for locating mission/quest data.

- `MissionActivityData.user.3`, which appears to only contain _story missions_, both main-story and side-story.
- In every `Mission######` directory, if a directory exists named `_Quest`, a file with the suffix `_QuestData.user.3`
  can be read to load non-story missions, such as assignments and optional quests.

At the time of writing, neither of those appear to include event quests. Additionally, I've found at least one optional
quest that isn't present in the quest data file: *Proof of a Hero*. I'm not going to let that stop me from moving
forward with the missions I *can* find, but I want to document it as a known issue.

**I AM SLOWLY LOSING MY MIND.**

QuestData files are _exclusively_ assignments. The MsData file they match to can be for different things, based on the
`_MissionTypeSerial` property in the MsData file.

MissionActivityData contains all main story and side (optional) quests. They MAY also include a QuestData file, BUT
if they do, that's just an extra assignment that unlocks as part of it (on completion I guess?). **Note**, optional
quests also seem to have a QuestData file, which should not be parsed as an assignment.

I have no answer for event quests. They've gotta be in some mysterious part of the PAK files that I just haven't found
yet. There's no reference to any event quests in any of the MsData or QuestData files I have right now.

Free Challenge quests are the same problem as event quests.

**Example files:**
- Main: Ms001030_MsData.json
- Side: Ms501010_MsData.json
- Optional: Ms103001_QuestData.json & Ms103001_MsData.json
- Assignment: Ms001030_QuestData.json

So basically:
- Main and side missions are the missions listed in MissionActivityData.
- Assignments are derived from the QuestData file that belong to a main or side mission.
- Optional missions are all other Mission###### directories that contain a QuestData file.

For optional missions, it looks like they're all listed in FreeQuestSortData. That's probably a great entrypoint, in
the same way that MissionActivityData is the entrypoint for main/side missions.

QuestOpenData appears to hold unlock conditions for optional quests. The `_MissionId` property is the mission being
unlocked, `_ConditionType` indicates the type of unlock condition (as listed in the engine enum snippet below), and
`_ConditionMission` or `_ConditionEnemy` defines what must be done to unlock it (which one is based on `_ConditionType`).

```c++
namespace app::QuestOpenType {
    enum TYPE_Fixed {
        MISSION_CLEAR = 0,
        ENEMY_HUNT = 1,
        ENEMY_HUNT_EX = 2,
    };
}
```

## Entrypoints
Both `MissionActivityData.user.3` and `*_QuestData.user.3` files will need to be used to parse out mission data.

### MissionActivityData.user.3
Each top-level entry in this file (the immediate descendent object of the array) is one story mission, with its
chapter and subchapter defined in `_ParentChapterId` and `_ChildChapterId`. Note that not all records will have a
defined chapter, indicated by `-1` in both fields. As far as I can tell, those are _always_ tagged with either
`_IsSideMission` or `_IsExtraMission` set to `true`.

In each mission record, the `_SettingMissionDataList` seems to define an array of mission steps (almost like
"checkpoints" along the mission chain?). Each element in this array contains a `_MissionIDSerial` which is used to
identify which of the `Mission######/` directories to check for that step's data. Each record also contains a
`_SettingObjectiveData` property, which is an array of the steps within that sub-mission (don't know if I like calling
it that, but whatever) and the text that should be displayed in the quest guide.

# Missions (old)
**This entire section (and its subheadings) are being kept for the sake of maintaining history, but are no longer
relevant. See [Missions](#missions) for current analysis.**

Most files seem to be found under `Mission/_UserData` and `Mission/Mission<id>` where `<id>` is a unique numeric
identifier. It is _not_ however, the same as the engine ID defined in the enums file.

There are two general-purpose data files that should make a good entry point, both located in `Mission/_UserData`:
- `MissionActivityData.user.3`, which seems to contain mission labels, flavor text, objective text, etc.
- `QuestOpenData.user.3`, which appears to hold unlock conditions.

`FreeQuestSortData.user.3` and `MainStorySortData.user.3`, both in the same directory, will also be useful to give
sorting hints.

There's also `HunterRankCupData.user.3`, which lists rank gate conditions, e.g. which mission needs to be cleared to
allow the player to rank up past HR40. There's a field that gives a name to the flag, and I think I could infer what
ranks the player is allowed to access based on that, but I'm certain there has to be another file that gives us that
info in a more machine-friendly format.

## Entrypoints
I think [Walking `Mission/Mission*` directories](#walking-missionmission-directories) is going to be the best approach.

I think parsing will need to be done in two passes: an initial pass to parse each directory into an enum that indicates
if the directory is a Mission or a Task. Missions are the directories that contain a QuestData file (MH tends to refer
to them as missions, so I think that makes more sense). Tasks are the directories without one, and are basically the
different steps the player takes while completing the mission. Then, the second pass can merge the Task entries into
their corresponding Mission entry, so the file merged file contains only Missions at the top level, and all their
(ordered) tasks within that top-level object.

### MissionActivityData.user.3
This appears to have a list of every mission, their title and description UUIDs, and UUIDs for each of the quest steps.
We can just walk the file and get a list of every mission, _however_: there's no easy way to directly relate them
back to the mission ID directories in `Mission/` (e.g. `Mission/Mission001030`).

### QuestOpenData.user.3
I originally thought this would be a good entrypoint, but aside from also listing every mission, we get even less data
to work with, and still have no way to relate entries to their mission directory.

### Walking `Mission/Mission*` directories
Instead of trying to find some perfect file to act as an entrypoint, we could just iterate over every directory matching
the glob `Mission/Mission*`. Each directory has an `Ms#####_MsData.user.3` file that contains information about the
quest, as well as its game ID, which we could use to look up entries from `MissionActivityData.user.3` later on during
merging.

The MsData files aren't the right target, it'll be the QuestData files. Far as I can tell, any directory with one of
those files is the starting point for a quest chain within that mission, e.g. `Mission001030/Ms001030_QuestData.user.3`
is the first quest belonging to "Slimy Situation", and subsequent entries in the chain indicate that by putting the
previous quest's ID in the `_OpenFlagMissionList` field (in the quest's MsData file). So, for "Slimy Situation", the
files for the next step in the chain would be in the `Mission001040` directory, since its MsData file has the ID
of our entry point file in `_OpenFlagMissionList`. To find the next step, we simply repeat that process: search through
the MsData files until we find one with Mission001040's ID in `_OpenFlagMissionList`, continuing until we hit a
directory that contains a QuestData file.

I'm _certain_ this will get more complicated for side/event quests, but. I'll burn that bridge when I get there.