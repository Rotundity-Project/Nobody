# Script Guide

## 1. Location
Place script files under `example_scripts/` or any local path, then load with `Custom` in UI.

## 2. File Type
- JSON
- UTF-8 recommended

## 3. Root Schema
```json
{
  "id": "string",
  "name": "string",
  "script_type": "Custom",
  "world_setting": { ... },
  "initial_state": { ... }
}
```

## 4. Required Fields
- `world_setting.cultivation_realms` must not be empty
- `world_setting.locations` must not be empty
- `initial_state.starting_location` must match one `locations[].id`
- `initial_state.starting_age` must be between `10` and `100`

## 5. Enum Values
- `script_type`: `Custom` | `RandomGenerated` | `ExistingNovel`
- `element`: `Fire` | `Water` | `Wood` | `Metal` | `Earth`
- `grade`: `Heavenly` | `Pseudo` | `Triple` | `Double`

## 6. Minimal Valid Example
```json
{
  "id": "minimal_script",
  "name": "Minimal Script",
  "script_type": "Custom",
  "world_setting": {
    "cultivation_realms": [
      { "name": "Qi Condensation", "level": 1, "sub_level": 0, "power_multiplier": 1.0 }
    ],
    "spiritual_roots": [
      { "element": "Fire", "grade": "Double", "affinity": 0.7 }
    ],
    "techniques": [],
    "locations": [
      { "id": "village", "name": "Village", "description": "A quiet start point", "spiritual_energy": 1.0 }
    ],
    "factions": []
  },
  "initial_state": {
    "player_name": "Player",
    "player_spiritual_root": { "element": "Fire", "grade": "Double", "affinity": 0.7 },
    "starting_location": "village",
    "starting_age": 16
  }
}
```

## 7. Example Files
- `example_scripts/sect_apprentice.json`
- `example_scripts/wandering_sword.json`
- Existing references:
  - `example_scripts/test_script.json`
  - `example_scripts/test_script_cn.json`
