# Hollow Knight & Silksong Save Monitor

A tool for monitoring and manipulating save files for Hollow Knight and Hollow
Knight: Silksong.

## Features

-   Real-time monitoring of save file changes
-   Save file decoding/encoding to/from JSON format and `.dat`
-   Support for both Hollow Knight and Silksong
-   Change tracking with detailed diffs
-   Cross-platform support (Windows, macOS, Linux)

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases
page](https://github.com/zohnannor/hk-save-logger/releases).

### Building from Source

1. Install Rust from https://rustup.rs/
2. Clone the repository:

    ```bash
    git clone https://github.com/zohnannor/hk-save-logger.git
    cd hk-save-logger
    ```

3. Build the project:

    ```bash
    bash
    cargo build --release
    ```

    The binary will be available at `target/release/hk-save-logger`

## Usage

### Basic Monitoring

Monitor Hollow Knight save slot 1:

```bash
hk-save-logger hollow-knight 1
# or
hk-save-logger hk 1
```

Monitor Silksong save slot 2:

```bash
hk-save-logger silksong 2
# or
hk-save-logger ss 2
```

Monitor a specific save file:

```bash
hk-save-logger --path /path/to/your/save/user1.dat
```

### Encoding/Decoding

Decode a save file to JSON:

```bash
hk-save-logger --path /path/to/save.dat
# or
hk-save-logger ss 1
```

Encode a modified JSON file back to save format (`.dat`):

```bash
hk-save-logger --path /path/to/save.dat --encode
```

Command Line Options

```text
Usage: hk-save-logger [OPTIONS] [GAME] [SAVE]

Arguments:
  [GAME]  Game to parse (hollow-knight/hk or silksong/ss) [possible values: hollow-knight, silksong]
  [SAVE]  Save slot to parse (1-4)

Options:
      --encode       Decode save file (default) or encode (it won't place the file for you, you have to do it manually, this is done to prevent data loss. Rename file to user1.dat (or whatever number you need) and place it in the correct folder)
      --path <PATH>  Path to save file (default: auto-detect)
  -h, --help         Print help
  -V, --version      Print version
```

## Save File Locations

The tool automatically detects save file locations for both games and both Steam
and GOG:

-   Windows: `%USERPROFILE%\AppData\LocalLow\Team Cherry\`
-   macOS: `~/Library/Application Support/`
-   Linux: `~/.config/unity3d/Team Cherry/`

## Output Files

The tool creates (or appends to) two files:

-   `{game}-{save}.json` - JSON representation of the save file
-   `{game}-{save}.log` - Log of all changes detected

Example output:

```json
[2025-09-10 13:50:19.1053902 +03:00:00] Change #10 detected:
  playerData.HeroCorpseMarkerGuid: null -> "HtdHrq1NO02NyYPJ4NjRpA=="
  playerData.FisherWalkerDirection: false -> true
  playerData.FisherWalkerIdleTimeLeft: -0.00288200425 -> 48.22196
  playerData.FisherWalkerTimer: 44.7499161 -> -0.00561722228
  playerData.HeroCorpseMoneyPool: 0 -> 1013
  playerData.HeroCorpseScene: "" -> "Song_Tower_01"
  playerData.HeroDeathScenePos.x: 67.47 -> 53.51
  playerData.HeroDeathScenePos.y: 16.84 -> 102.620064
  playerData.HeroDeathSceneSize.x: 124.0 -> 130.0
  playerData.HeroDeathSceneSize.y: 56.0 -> 140.0
  playerData.IsSilkSpoolBroken: false -> true
  playerData.LastSetFieldName: "HasSeenGeoMid" -> "encounteredLaceTower"
  playerData.QuestCompletionData.savedData[28].Data.IsCompleted: false -> true
  playerData.QuestCompletionData.savedData[28].Data.WasEverCompleted: false -> true
  playerData.QuestCompletionData.savedData[43]: null -> {"Data":{"CompletedCount":0,"HasBeenSeen":false,"IsAccepted":true,"IsCompleted":false,"WasEverCompleted":false},"Name":"Citadel Ascent Lift"}
  playerData.QuestPaneHasNew: false -> true
  playerData.UnlockedMelodyLift: false -> true
  playerData.encounteredLaceTower: false -> true
  playerData.environmentType: 4 -> 0
  playerData.geo: 1013 -> 0
  playerData.health: 3 -> 0
  playerData.playTime: 160898.813 -> 161115.266
  playerData.prevHealth: 6 -> 8
  playerData.scenesVisited[682]: null -> "Song_Tower_01"
  playerData.silk: 3 -> 0

[2025-09-10 13:51:52.2267669 +03:00:00] Change #11 detected:
  playerData.FisherWalkerDirection: true -> false
  playerData.FisherWalkerIdleTimeLeft: 48.22196 -> -0.00368880527
  playerData.FisherWalkerTimer: -0.00561722228 -> 12.2016125
  playerData.InvPaneHasNew: true -> false
  playerData.LastSetFieldName: "encounteredLaceTower" -> "QuestPaneHasNew"
  playerData.MaskMakerQueuedUnmasked2: false -> true
  playerData.QuestCompletionData.savedData[43].Data.HasBeenSeen: false -> true
  playerData.QuestPaneHasNew: true -> false
  playerData.ToolEquips.savedData[6].Data.Slots[3].EquippedTool: "Revenge Crystal" -> "Poison Pouch"
  playerData.belltownCrowd: 4 -> 1
  playerData.bonetownCrowd: 6 -> 5
  playerData.currentInvPane: 4 -> 1
  playerData.environmentType: 0 -> 4
  playerData.grubFarmerTimer: 1493.97046 -> 1714.77307
  playerData.halfwayCrowd: 1 -> 3
  playerData.hazardRespawnFacing: 0 -> 1
  playerData.health: 0 -> 8
  playerData.mapperAway: false -> true
  playerData.pilgrimGroupBonegrave: 2 -> 1
  playerData.pilgrimGroupGreymoorField: 3 -> 2
  playerData.pilgrimGroupShellgrave: 3 -> 2
  playerData.pilgrimRestCrowd: 1 -> 3
  playerData.pinstressInsideSitting: false -> true
  playerData.playTime: 161115.266 -> 161176.031
  playerData.silk: 0 -> 1
```

## Bugs

If you find any bugs, please [open an
issue](https://github.com/zohnannor/hk-save-logger/issues/new) and describe your
problem.

## Known Issues

-   Insertion of a new value into the middle of an array is rendered as a change
    to all the subsequent values

## License

This project is licensed under either of

-   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
    http://www.apache.org/licenses/LICENSE-2.0)
-   MIT license ([LICENSE-MIT](LICENSE-MIT) or
    http://opensource.org/licenses/MIT)

at your option.

#### Disclaimer

Yes, some parts (json diff and github action) are generated by DeepSeek bc I
can't be bothered
