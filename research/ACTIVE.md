- [Missions](#missions)
  - [Entrypoints](#entrypoints)
    - [MissionActivityData.user.3](#missionactivitydatauser3)
    - [QuestOpenData.user.3](#questopendatauser3)
    - [Walking `Mission/Mission*` directories](#walking-missionmission-directories)

# Missions
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