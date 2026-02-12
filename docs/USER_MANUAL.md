# Nobody User Manual

## 1. Overview
Nobody is a text-based cultivation simulator built with Tauri + Vue.
You can start from:
- custom JSON scripts
- AI-generated random scripts
- existing novel import (text + character selection)

## 2. Requirements
- Node.js 20+
- Rust toolchain (stable)
- Windows/macOS/Linux desktop environment for Tauri

## 3. Install And Run
```bash
npm install
npm run tauri:dev
```

Build:
```bash
npm run tauri:build
```

## 4. Basic Game Flow
1. Open app and click `New Game`.
2. Choose one script source:
- `Custom`: load a local `.json` script.
- `Random Generated`: generate by LLM.
- `Existing Novel`: select `.txt/.md`, then choose a character.
3. Enter game screen.
4. Pick options or enter free text actions.
5. Save/load progress with save slots.

## 5. Main UI
- Main Menu: new game, load game, settings.
- Script Selector: choose source and import path.
- Game View: story text, options, free-text action input.
- Character Panel: name, realm, spiritual root, lifespan, combat power.
- Save/Load Dialog: slot operations.
- Novel Exporter: generate and export novel after playthrough.

## 6. Save/Load
- Save slots are indexed by `slot_id`.
- Recommended range is `1..99`.
- Save and load are available through in-game dialogs.

## 7. LLM Features
If configured, LLM is used for:
- random script generation
- NPC decision generation
- free-text intent parsing
- plot text generation
- novel generation

Without valid LLM config, some features fallback to local behavior.

## 8. Common Issues
- Script load failed:
  - validate JSON structure
  - ensure `initial_state.starting_location` exists in `world_setting.locations`
- Novel import failed:
  - use `.txt` or `.md`
  - make sure parser can extract character names
- LLM timeout:
  - verify endpoint/model/api key
  - retry with smaller request load

## 9. Related Docs
- `docs/SCRIPT_GUIDE.md`
- `docs/API.md`
- `CONTRIBUTING.md`
