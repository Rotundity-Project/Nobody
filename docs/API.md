# Nobody Tauri API 文档

本文档列出前端通过 Tauri 调用的主要命令，源码位置：`src-tauri/src/tauri_commands.rs`。

## 1. LLM 配置

### `set_llm_config(input)`

- 入参：
  - `endpoint: string`
  - `apiKey: string`
  - `model: string`
  - `maxTokens: number`
  - `temperature: number`
- 返回：`string`

### `clear_llm_config()`

- 返回：`string`

### `get_llm_config_status()`

- 返回：配置状态对象

### `test_llm_connection()`

- 返回：`string`（测试响应）

## 2. 游戏生命周期

### `initialize_game({ script })`

- 入参：`Script`
- 返回：`GameState`

### `initialize_plot()`

- 返回：`PlotState`

### `get_game_state()`

- 返回：`GameState`

### `get_plot_state()`

- 返回：`PlotState`

### `update_plot_settings({ settings })`

- 入参：`PlotSettings`
- 返回：`PlotState`

## 3. 玩家行动

### `execute_player_action({ action })`

- 入参：`PlayerAction`
- 返回：`string`（剧情文本）

### `get_player_options()`

- 返回：`PlayerOption[]`

## 4. 存档系统

### `save_game({ slotId })`

- 入参：`slotId: number`
- 返回：`void`

### `load_game({ slotId })`

- 入参：`slotId: number`
- 返回：`GameState`

### `list_save_slots()`

- 返回：`SaveInfo[]`

## 5. 剧本与导入

### `load_script({ scriptPath })`

- 入参：本地 `.json` 路径
- 返回：`Script`

### `generate_random_script()`

- 返回：`Script`

### `parse_novel_characters({ novelPath })`

- 入参：本地 `.txt` / `.md` 路径
- 返回：`string[]`

### `load_existing_novel({ novelPath, selectedCharacter })`

- 入参：
  - `novelPath: string`
  - `selectedCharacter: string`
- 返回：`Script`

## 6. 导出与扩展

### `generate_novel({ title })`

- 入参：`title: string`
- 返回：小说结构对象

### `export_novel({ ... })`

- 入参与返回以当前命令定义为准。

## 7. 错误处理约定

- 命令统一返回 `Result<_, String>`。
- 前端应展示可读错误，并保留重试入口。
- 涉及 LLM 的命令应处理超时与降级提示。
