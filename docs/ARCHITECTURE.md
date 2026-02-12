# Nobody 架构文档

## 1. 架构概览
Nobody 采用 `Tauri2 + Vue3 + Rust` 的桌面应用架构：
- 前端：Vue3 + Pinia + Vue Router，负责界面与交互流程。
- 后端：Rust 模块化服务，负责游戏核心逻辑、存档、LLM、NPC、剧情推进。
- 通信层：Tauri command（`invoke`）作为前后端边界。

## 2. 分层设计
### 2.1 表现层（Frontend）
- 目录：`src/components`、`src/stores`、`src/router`
- 关键职责：
  - 页面与组件渲染（主菜单、剧本选择、游戏主界面、存档、小说导出）
  - 用户输入采集（选项与自由文本）
  - 通过 Pinia Store 统一状态调用后端命令

### 2.2 应用层（Tauri Commands）
- 文件：`src-tauri/src/tauri_commands.rs`
- 关键职责：
  - 参数校验（路径、slot、LLM 配置）
  - 调用领域服务（GameEngine / ScriptManager / NovelGenerator 等）
  - 统一错误消息返回 `Result<_, String>`

### 2.3 领域层（Rust Core）
- 关键模块：
  - `game_engine.rs`：游戏全局状态与核心流程编排
  - `plot_engine.rs`：剧情推进与行动处理
  - `numerical_system.rs`：数值系统与战斗/成长逻辑
  - `npc_engine.rs` + `memory_manager.rs`：NPC 决策与记忆
  - `script_manager.rs` + `script.rs`：剧本加载、验证、随机/小说导入
  - `save_load.rs`：存档读写与校验
  - `novel_generator.rs` + `event_log.rs`：事件记录与小说生成
  - `llm_service.rs` + `prompt_builder.rs` + `response_validator.rs`：LLM 调用链路

## 3. 关键数据流
### 3.1 开局流程（以自定义剧本为例）
1. 前端调用 `load_script`
2. 后端 `ScriptManager` 解析并校验 `Script`
3. 前端调用 `initialize_game`
4. `GameEngine` 基于 `Script` 初始化 `GameState`
5. 前端调用 `initialize_plot`，创建初始 `PlotState`

### 3.2 玩家行动流程
1. 前端提交 `execute_player_action`
2. `PlotEngine` 校验并处理行动
3. `NumericalSystem` 计算属性变化
4. `GameEngine` 更新状态并记录事件
5. 前端再拉取 `get_game_state` / `get_plot_state` 刷新 UI

### 3.3 存档流程
1. 前端调用 `save_game(slot_id)`
2. `GameEngine` 委托 `SaveLoadSystem` 写入 JSON
3. 加载时调用 `load_game(slot_id)` 并恢复状态

## 4. 模块关系图（文本）
```text
[Vue Components]
      |
      v
[Pinia Store]
      |
      v  invoke()
[Tauri Commands]
      |
      +--> [ScriptManager] --> [NovelParser / LLMService]
      |
      +--> [GameEngine] ---> [PlotEngine] ---> [NumericalSystem]
      |         |
      |         +-----------> [NPCEngine] ---> [MemoryManager]
      |         |
      |         +-----------> [SaveLoadSystem]
      |         |
      |         +-----------> [EventLog] ---> [NovelGenerator]
      |
      +--> [LLM Runtime Config / Validation]
```

## 5. 状态模型
- 前端状态（Pinia）：
  - `currentScript`
  - `gameState`
  - `plotState`
  - `isLoading` / `error`
- 后端核心状态：
  - `GameState`：角色、世界、时间、事件
  - `PlotState`：当前场景、历史、章节、可选项
  - 通过 `Mutex<GameEngine>` 在 Tauri 进程内托管

## 6. 设计原则
- 单一职责：前端不承载核心规则，规则统一在 Rust 侧。
- 边界清晰：所有跨层调用必须通过 Tauri command。
- 可回退：LLM 失败时提供 fallback 路径，保证可玩性。
- 可验证：关键逻辑配套单元测试/属性测试。

## 7. 可演进方向
- 将 command 校验抽成独立 validator 模块，减少重复。
- 引入更明确的 CQRS（查询/命令分离）以降低状态耦合。
- 对 LLM 链路增加统一 tracing 与指标采集（耗时、重试、缓存命中率）。
