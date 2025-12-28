- [About](#about)
  - [Requirements](#requirements)
  - [Pipeline](#pipeline)
- [Tools](#tools)
- [Credits](#credits)

# About
The goal of this project is to "glue" several other tools together in order to get sane JSON files for data objects in
Wilds. This repo is used by the [MHDB Wilds Project](https://docs.wilds.mhdb.io) as it's primary data source.

**If you're just looking for game data**, you don't need to build the merged files yourself. The most recent version of
all the merged data files are available in
[`/output/merged`](https://github.com/LartTyler/mhdb-wilds-data/tree/main/output/merged).

## Requirements
- Rust

## Pipeline
1. Use `/tools/ree-pak-gui` to extract all `.pak` files into `/data`. Only `.user.*` and `.msg.*` files need to be
   extracted.
2. Run `/extract.bat` to convert the relevant `.user.3` and `.msg.23` files to JSON dumps.
3. Run `/merge.bat` to convert the raw JSON dumps into a merged JSON format.

# Tools
See [`/tools/README.md`](tools/README.md) for credits and information on the various tools used to make this project
possible.

# Credits
- [REMSG_Converter by dtlnor](https://github.com/dtlnor/REMSG_Converter)
- [ree-pak-gui by eigeen](https://github.com/eigeen/ree-pak-gui)
- [MHWS-Editor by Synthlight](https://www.nexusmods.com/monsterhunterwilds/mods/32)

