# Nobody Tauri API

This document lists public Tauri commands from `src-tauri/src/tauri_commands.rs`.

## 1. LLM Config
### `set_llm_config(input)`
- Input:
  - `endpoint: string`
  - `apiKey: string`
  - `model: string`
  - `maxTokens: number`
  - `temperature: number`
- Return: `string`

### `clear_llm_config()`
- Return: `string`

### `get_llm_config_status()`
- Return: runtime config status object

### `test_llm_connection()`
- Return: `string` (model response text)

## 2. Game Lifecycle
### `initialize_game({ script })`
- Input: `Script`
- Return: `GameState`

### `initialize_plot()`
- Return: `PlotState`

### `get_game_state()`
- Return: `GameState`

### `get_plot_state()`
- Return: `PlotState`

### `update_plot_settings({ settings })`
- Input: `PlotSettings`
- Return: `PlotState`

## 3. Player Actions
### `execute_player_action({ action })`
- Input: `PlayerAction`
- Return: `string` (new plot text segment)

### `get_player_options()`
- Return: `PlayerOption[]`

## 4. Save/Load
### `save_game({ slotId })`
- Input: `slotId: number` (`1..99`)
- Return: `void`

### `load_game({ slotId })`
- Input: `slotId: number`
- Return: `GameState`

### `list_save_slots()`
- Return: `SaveInfo[]`

## 5. Script Import/Generation
### `load_script({ scriptPath })`
- Input: local `.json` file path
- Return: `Script`

### `generate_random_script()`
- Return: `Script`

### `parse_novel_characters({ novelPath })`
- Input: local `.txt` or `.md` file path
- Return: `string[]`

### `load_existing_novel({ novelPath, selectedCharacter })`
- Input:
  - `novelPath: string`
  - `selectedCharacter: string`
- Return: `Script`

## 6. Novel
### `generate_novel({ title })`
- Input: `title: string`
- Return: `Novel`

### `export_novel({ novel, outputPath })`
- Input:
  - `novel: Novel`
  - `outputPath: string` (`.txt`)
- Return: `void`

## 7. Error Conventions
- Commands return `Result<_, String>` on backend side.
- Frontend should handle message text as user-visible failure reason.
