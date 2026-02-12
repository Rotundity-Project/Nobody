# 剧本编写指南

## 1. 文件放置位置

推荐将剧本放在 `example_scripts/`，也可放在任意本地目录，然后在 UI 中通过 `自定义剧本` 加载。

## 2. 文件格式

- 必须为 JSON
- 推荐 UTF-8 编码

## 3. 顶层结构

```json
{
  "id": "string",
  "name": "string",
  "script_type": "Custom",
  "world_setting": {},
  "initial_state": {}
}
```

## 4. 必填约束

- `world_setting.cultivation_realms` 不能为空
- `world_setting.locations` 不能为空
- `initial_state.starting_location` 必须匹配 `locations[].id`
- `initial_state.starting_age` 必须在 `10..100` 之间

## 5. 常见枚举值

- `script_type`: `Custom` | `RandomGenerated` | `ExistingNovel`
- `element`: `Fire` | `Water` | `Wood` | `Metal` | `Earth`
- `grade`: `Heavenly` | `Pseudo` | `Triple` | `Double`

## 6. 最小可用示例

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

## 7. 参考样例

- `example_scripts/sect_apprentice.json`
- `example_scripts/wandering_sword.json`
- `example_scripts/test_script.json`
- `example_scripts/test_script_cn.json`
