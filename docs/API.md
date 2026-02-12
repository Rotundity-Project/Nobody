# Nobody Tauri API 文档

本文档汇总 `src-tauri/src/tauri_commands.rs` 暴露的主要 Tauri 命令。

## 1. LLM 配置

### `set_llm_config(input)`
- 入参:
  - `endpoint: string`
  - `apiKey: string`
  - `model: string`
  - `maxTokens: number`
  - `temperature: number`
- 返回: `string`

### `clear_llm_config()`
- 返回: `string`

### `get_llm_config_status()`
- 返回: 运行时配置状态对象

### `test_llm_connection()`
- 返回: `string`（模型返回文本）

## 2. 游戏生命周期

### `initialize_game({ script })`
- 入参: `Script`
- 返回: `GameState`

### `initialize_plot()`
- 返回: `PlotState`

### `get_game_state()`
- 返回: `GameState`

### `get_plot_state()`
- 返回: `PlotState`

### `update_plot_settings({ settings })`
- 入参: `PlotSettings`
- 返回: `PlotState`

## 3. 玩家行动

### `execute_player_action({ action })`
- 入参: `PlayerAction`
- 返回: `string`（新剧情文本片段）

### `get_player_options()`
- 返回: `PlayerOption[]`

## 4. 存档与读档

### `save_game({ slotId })`
- 入参: `slotId: number`（`1..99`）
- 返回: `void`

### `load_game({ slotId })`
- 入参: `slotId: number`
- 返回: `GameState`

### `list_save_slots()`
- 返回: `SaveInfo[]`

## 5. 剧本导入与生成

### `load_script({ scriptPath })`
- 入参: 本地 `.json` 文件路径
- 返回: `Script`

### `generate_random_script()`
- 返回: `Script`

### `parse_novel_characters({ novelPath })`
- 入参: 本地 `.txt` 或 `.md` 文件路径
- 返回: `string[]`

### `load_existing_novel({ novelPath, selectedCharacter })`
- 入参:
  - `novelPath: string`
  - `selectedCharacter: string`
- 返回: `Script`

## 6. 小说生成与导出

### `generate_novel({ title })`
- 入参: `title: string`
- 返回: `Novel`

### `export_novel({ novel, outputPath })`
- 入参:
  - `novel: Novel`
  - `outputPath: string`（`.txt`）
- 返回: `void`

## 7. 错误处理说明

- 所有命令错误统一以字符串返回给前端。
- 输入参数在命令层会进行校验（如槽位范围、文件路径、扩展名）。
- 依赖外部服务（LLM）时建议前端使用超时包装并展示加载反馈。
