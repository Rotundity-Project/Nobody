# Nobody 架构文档

## 1. 架构概览
Nobody 采用 `Tauri 2 + Vue 3 + Rust` 桌面应用架构：

- 前端：Vue 3 + Pinia + Vue Router，负责 UI 与交互编排。
- 后端：Rust 领域模块，负责剧情推进、规则计算、存档与 LLM 集成。
- 通信：Tauri commands（`invoke`）作为前后端边界。

## 2. 分层设计

### 2.1 前端层（`src/`）

- `views/`：路由级页面容器
- `components/`：领域组件
- `composables/`：交互编排逻辑
- `shared/ui/`：基础 UI 组件
- `stores/`：Pinia 状态管理

职责：展示状态、收集输入、调用后端命令、渲染反馈。

### 2.2 命令层（`src-tauri/src/tauri_commands.rs`）

职责：

- 参数校验
- 领域服务编排
- 错误统一映射为可读消息

### 2.3 领域层（`src-tauri/src/*.rs`）

关键模块：

- `game_engine.rs`：全局状态与流程编排
- `plot_engine.rs`：剧情段落生成与推进
- `plot_consistency.rs`：一致性校验/修复
- `numerical_system.rs`：数值系统
- `npc_engine.rs`、`memory_manager.rs`：NPC 决策与记忆
- `save_load.rs`：存档读写
- `llm_service.rs`：LLM 调用

## 3. 关键数据流

### 3.1 开局流程

1. 前端加载脚本（`load_script` / `generate_random_script`）
2. 初始化游戏（`initialize_game`）
3. 初始化剧情（`initialize_plot`）
4. 前端进入 `GameRuntimeView`

### 3.2 玩家行动流程

1. 前端提交 `execute_player_action`
2. 后端推进剧情并更新游戏状态
3. 前端拉取 `get_game_state` / `get_plot_state`
4. UI 刷新：剧情、选项、状态、通知

### 3.3 存档流程

1. 保存：`save_game(slotId)`
2. 读取：`load_game(slotId)`
3. 列表：`list_save_slots()`

## 4. 状态模型

前端（Pinia）：

- `currentScript`
- `gameState`
- `plotState`
- `isLoading` / `error`

后端核心：

- `GameState`：角色/世界/时间/事件
- `PlotState`：场景/章节/可选项/交互状态

## 5. 设计原则

- 单一事实源：状态统一由后端核心与前端 store 管理。
- 明确边界：跨层调用必须走 Tauri command。
- 可回退：LLM 失败时必须有规则兜底。
- 可回归：关键路径保持测试覆盖。

## 6. 演进方向

- 完善 command 参数与返回结构的统一 schema。
- 加强 LLM 链路观测（耗时、重试、降级来源）。
- 继续沉淀 shared/ui 与 composables，降低页面耦合。
