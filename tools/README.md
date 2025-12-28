- [Tools](#tools)
  - [ree-pak-gui](#ree-pak-gui)
  - [REMSG\_Converter](#remsg_converter)
  - [MHWS-Editor](#mhws-editor)
- [Rust Libraries](#rust-libraries)
  - [rslib](#rslib)
  - [parser](#parser)
- [Rust Applications](#rust-applications)
  - [extractor](#extractor)
  - [merger](#merger)
- [Credits](#credits)

# Tools
## ree-pak-gui
[ree-pak-gui](https://github.com/eigeen/ree-pak-gui) is used to extract data files from the `.pak` files that ship
with the game. At the moment, only `.user.3` and `.msg.23` are extracted, since those are the data files and language
files respectively.

## REMSG_Converter
[REMSG_Converter](https://github.com/dtlnor/REMSG_Converter) can be used to convert `.msg.*` files to JSON dumps
containing translations. Contents can be matched via the `guid` field, which contains a UUID that the data files use
to reference the relevant translations. Entries in the `content` array are an ordered list of the term of phrase in
every language that RE Engine supports; empty strings indicate that the language isn't supported by Wilds. See the
`LanguageCode` enum in [`/tools/rslib/src/formats/msg.rs`](rslib/src/formats/msg.rs) for a list of supported languages
in the order they appear in `content`.

Note that certain values in the `content` array indicate that the translations are not valid, usually because the thing
they belong to hasn't been officially added to the game yet, or has since been removed. Some examples of this are the
strings "-", "---", and "#Rejected#".

## MHWS-Editor
[MHWS-Editor](https://www.nexusmods.com/monsterhunterwilds/mods/32) isn't part of the normal toolchain, but is
incredibly useful for research. It's a `.user.3` file browser and editor that supports almost all data files, and has
been instrumental in getting the project to where it is now.

# Rust Libraries
## rslib
`rslib` contains common code shared with the [`extractor`](#extractor) and [`merger`](#merger) projects, such as
descriptor objects for the configuration files, and utility methods for interacting with the JSON dumps from `.user.3`
and `.msg.23` files.

## parser
The `parser` library is a Rust implementation of an RSZ parser for `.user.3` data files. It is used by several utilities
in [`rslib`](#rslib) to read data from data files so they can be dumped to JSON files for merging.

Note that this library is built specifically to support the needs of this project, and may not be suitable for general
use. I do plan on improving it over time, however, and would be very happy to discuss adding support for other file
types. Once the Wilds API project is feature-complete, I will be moving this library out of this project to be it's own
standalone project so it can be published as a Rust crate.

# Rust Applications
## extractor
The `extractor` application is used to generate JSON dumps of data and language files so they can be easily parsed and
merged later on by the [`merger`](#merger) application. The dumps created by `extractor` are, basically, as-is
representations of the data in the data and language files, and no effort is made by `extractor` to fill in placeholder
data or link UUID references.

## merger
The `merger` application takes the JSON dumps created by [`extractor`](#extractor) and combines related files into a
clean JSON representation. Where possible, UUID relations are resolved (or are moved to their own dedicated files if
the referenced data is useful outside of the context that refereces it, such as item data) and placeholders in
translation data are filled in (where possible).

# Credits
- [REMSG_Converter by dtlnor](https://github.com/dtlnor/REMSG_Converter)
- [ree-pak-gui by eigeen](https://github.com/eigeen/ree-pak-gui)
- [MHWS-Editor by Synthlight](https://www.nexusmods.com/monsterhunterwilds/mods/32)