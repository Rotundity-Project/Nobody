# Nobody Agent Framework V1 实现方案

更新时间: 2026-04-07
状态: 提案
目标版本: V1

## 1. 背景

`Nobody` 当前已经具备较完整的“Agent-ready”基础设施：

- 运行时状态：`src-tauri/src/game_engine.rs`, `src-tauri/src/game_state.rs`, `src-tauri/src/plot_engine.rs`
- 世界知识与记忆：`src-tauri/src/world_registry.rs`, `src-tauri/src/memory_layers.rs`, `src-tauri/src/context_builder.rs`
- 校验与护栏：`src-tauri/src/plot_consistency.rs`, `src-tauri/src/entity_validator.rs`, `src-tauri/src/state_patch_validator.rs`, `src-tauri/src/numeric_guard.rs`
- 模型调用：`src-tauri/src/llm_service.rs`, `src-tauri/src/prompt_builder.rs`, `src-tauri/src/response_validator.rs`

当前缺少的不是单次生成能力，而是“带目标、带工具、带状态、带护栏”的连续决策能力。Agent Framework 的作用，就是把现有能力组织成可迭代的执行框架。

## 2. 设计目标

- 让剧情推进从“单次续写”升级为“目标驱动的分步决策”。
- 让重要 NPC 和世界状态具备更强的持续性与自主感。
- 保持规则优先，禁止 Agent 直接绕过引擎写入最终状态。
- 保持可观测、可回放、可测试，避免系统失控。
- 尽量复用现有模块，而不是重写整套后端。

## 3. 非目标

- V1 不追求多 Agent 自由协作。
- V1 不让 Agent 直接拥有 `GameState` 最终写权限。
- V1 不替换数值系统、战斗裁决、存档系统。
- V1 不要求前端大改；前端只增加少量诊断与模式开关。

## 4. 总体方案

V1 采用“单 Agent 编排 + 工具调用 + 规则护栏”的架构。

核心原则：

- Agent 只负责“提案”和“决策建议”。
- 引擎负责“校验”和“落地”。
- 所有状态变更都必须经过现有规则层。

建议的运行链路如下：

1. 前端继续调用 `execute_player_action`
2. `tauri_commands.rs` 在推进剧情前构建 Agent 输入上下文
3. `AgentOrchestrator` 调用单个 `DirectorAgent`
4. `DirectorAgent` 输出结构化 `AgentPlan`
5. Orchestrator 按计划调用内部工具
6. 工具结果进入一致性、数值、状态补丁校验
7. 通过校验后写回 `GameState`、`PlotState`、`WorldRegistry`
8. 生成 Agent Trace，供调试和 UI 展示

## 5. V1 推荐角色

### 5.1 Director Agent

职责：

- 判断当前回合剧情目标
- 决定本回合应该偏向铺垫、冲突、收束还是信息补全
- 选择需要调用的工具
- 输出候选剧情推进计划

边界：

- 不直接写最终文本段落
- 不直接修改 `GameState`
- 不直接决定数值结果

### 5.2 为什么先做 Director Agent

因为它最容易复用当前结构：

- 已有 `PlotState`
- 已有章节与交互状态机
- 已有上下文构建和一致性校验
- 已有快速模式/降级逻辑

相对而言，`NPC Agent` 和 `World Curator Agent` 更适合在 V1 稳定后进入 V2。

## 6. 模块落点

建议新增以下模块：

- `src-tauri/src/agent_types.rs`
- `src-tauri/src/agent_tools.rs`
- `src-tauri/src/agent_trace.rs`
- `src-tauri/src/agent_runtime.rs`
- `src-tauri/src/agent_prompts.rs`

建议复用/接入以下已有模块：

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/plot_engine.rs`
- `src-tauri/src/context_builder.rs`
- `src-tauri/src/world_registry.rs`
- `src-tauri/src/plot_consistency.rs`
- `src-tauri/src/state_patch_validator.rs`
- `src-tauri/src/llm_service.rs`

建议在 `src-tauri/src/lib.rs` 中注册新增诊断命令：

- `get_agent_trace`
- `clear_agent_trace`
- `get_agent_runtime_status`

## 7. 模块职责

### 7.1 `agent_types.rs`

定义 Agent 运行期的数据结构：

- `AgentMode`
- `AgentTurnInput`
- `AgentPlan`
- `AgentStep`
- `AgentToolCall`
- `AgentToolResult`
- `AgentProposal`
- `AgentGuardrailResult`

### 7.2 `agent_tools.rs`

定义 Agent 可调用的受控工具，不允许直接暴露原始状态写接口。

V1 工具建议：

- `ReadPlotSnapshot`
- `ReadWorldFacts`
- `ReadRecentEvents`
- `ReadCharacterProfile`
- `ReadReachableLocations`
- `GeneratePlotCandidate`
- `GenerateOptionHints`
- `ProposeWorldFactPatch`

### 7.3 `agent_trace.rs`

负责记录：

- 本回合 Agent 输入摘要
- 每一步决策理由
- 工具调用参数
- 工具返回摘要
- 最终是否采用
- 降级、拒绝原因

### 7.4 `agent_runtime.rs`

作为 orchestrator：

- 接收 `AgentTurnInput`
- 调用 LLM 生成 `AgentPlan`
- 逐步执行工具调用
- 整合结果并交给规则层
- 产出 `AgentOutcome`

### 7.5 `agent_prompts.rs`

维护 Agent 的系统提示与工具使用约束，避免把长 prompt 散落到 `tauri_commands.rs` 中。

## 8. 与现有链路的集成方式

### 8.1 集成点

V1 只接入主剧情推进链路：

- 入口：`src-tauri/src/tauri_commands.rs` 中的 `execute_player_action`

建议改造为：

1. 读取当前 `GameState`、`PlotState`、`WorldRegistry`
2. 构建 `AgentTurnInput`
3. 判断是否启用 Agent 模式
4. 若启用，则执行 `AgentRuntime::run_turn`
5. 将 `AgentProposal` 交给现有 `plot_engine` 和校验层
6. 若 Agent 失败，则走当前已有主链路或快速模式降级

### 8.2 不改动的核心边界

- 存档接口 `save_game` / `load_game` 不变
- 前端 `gameStore` 主调用方式不变
- `GameState` 和 `PlotState` 仍是单一事实源
- 真正的状态更新仍由后端引擎提交

## 9. V1 数据契约

### 9.1 Agent 输入

```ts
type AgentTurnInput = {
  turnId: string
  playerAction: {
    actionType: "free_text" | "selected_option"
    content: string
    selectedOptionId?: number | null
  }
  plotSnapshot: {
    chapterIndex: number
    interactionState: string
    currentScene: string
    recentParagraphs: string[]
  }
  playerSnapshot: {
    name: string
    realm: string
    location: string
    combatStatus?: Record<string, unknown>
  }
  worldFacts: Array<Record<string, unknown>>
  recentEvents: Array<Record<string, unknown>>
  reachableLocations: string[]
}
```

### 9.2 Agent 输出

```ts
type AgentPlan = {
  goal: "advance_plot" | "intensify_conflict" | "resolve_scene" | "enrich_world"
  rationale: string
  steps: AgentStep[]
}

type AgentStep = {
  kind: "read" | "generate" | "propose_patch"
  toolName: string
  input: Record<string, unknown>
}
```

### 9.3 最终提案

```ts
type AgentProposal = {
  plotCandidate?: {
    sceneSummary: string
    candidateParagraphs: string[]
    candidateOptions: string[]
    chapterGoalHit?: boolean
  }
  worldFactPatch?: Record<string, unknown>[]
  diagnostics: string[]
}
```

## 10. 护栏设计

Agent 接入成败的关键，不是模型能力，而是护栏是否足够强。

V1 至少保留 4 层护栏：

1. 输入护栏
   - Agent 只拿到裁剪过的上下文
   - 不直接暴露整个状态对象

2. 工具护栏
   - Agent 只能调用白名单工具
   - 工具入参与返回结构化

3. 提案护栏
   - Agent 只能生成 `Proposal`
   - Proposal 必须通过 schema 校验

4. 落地护栏
   - 使用现有 `plot_consistency.rs`
   - 使用现有 `state_patch_validator.rs`
   - 涉及实体时使用 `entity_validator.rs`
   - 涉及数值时使用 `numeric_guard.rs`

如果任一护栏失败，则：

- 记录 Trace
- 标记本回合 Agent 失败原因
- 自动回退到当前非 Agent 链路

## 11. 运行模式

建议引入运行时开关：

- `disabled`
- `observe_only`
- `assisted`

含义如下：

- `disabled`：完全关闭 Agent
- `observe_only`：Agent 只跑诊断和提案，不参与最终结果
- `assisted`：Agent 提案可参与最终结果，但必须通过全部护栏

V1 默认建议使用 `observe_only` 开发，稳定后切到 `assisted`。

## 12. 诊断与可观测性

建议新增 Trace 结构：

```ts
type AgentTrace = {
  turnId: string
  mode: string
  startedAt: number
  inputSummary: string
  steps: Array<{
    toolName: string
    ok: boolean
    summary: string
  }>
  guardrailResult: {
    accepted: boolean
    reasons: string[]
  }
  fallbackUsed: boolean
  elapsedMs: number
}
```

前端 V1 可只做轻量展示：

- 在信息中心增加 `Agent Trace` 调试页签
- 展示最近 10 次回合
- 仅在开发模式显示

## 13. 分阶段实施

### 阶段 0：准备

- 把 `tauri_commands.rs` 中的上下文拼装逻辑再收拢一层
- 统一主链路的诊断字段
- 修复现有 fallback 与测试失真问题

### 阶段 1：Agent 骨架

- 新增 `agent_types.rs`
- 新增 `agent_trace.rs`
- 新增 `agent_runtime.rs`
- 新增运行模式开关
- 在 `execute_player_action` 中接入 `observe_only`

交付标准：

- Agent 不影响现有结果
- 可记录输入、计划、工具调用、失败原因

### 阶段 2：受控提案

- 新增 `agent_tools.rs`
- Director Agent 生成结构化 `AgentPlan`
- 使用 `GeneratePlotCandidate` 工具产出候选段落
- 通过 `plot_consistency` 和 schema 校验

交付标准：

- 至少有一部分回合可以安全采用 Agent 结果
- 失败时自动回退到当前链路

### 阶段 3：世界补全

- 支持 `ProposeWorldFactPatch`
- 将通过校验的世界事实写入 `WorldRegistry`
- 强化章节总结与世界状态沉淀

交付标准：

- 世界事实能随剧情稳定增长
- 不破坏现有导出、地图、信息面板能力

### 阶段 4：扩展到 NPC

- 引入 `NpcIntentAgent` 或 `NpcDecisionModule`
- 仅对关键 NPC 启用
- 仍然走受控提案，不直接写最终状态

## 14. 测试策略

V1 推荐新增以下测试：

- `agent_runtime` 单元测试
  - 空输入
  - 非法计划
  - 工具失败
  - 护栏拒绝

- `execute_player_action` 集成测试
  - `disabled` 模式结果不变
  - `observe_only` 不改变现有剧情结果
  - `assisted` 在 Proposal 合法时能采用
  - Agent 失败时自动降级

- Trace 测试
  - 成功回合记录完整
  - 回退回合记录完整

- 属性测试
  - Agent 输出不会绕过状态不变量
  - Agent 输出的选项数仍满足现有约束

## 15. 风险与应对

### 风险 1：延迟明显上升

应对：

- 先用 `observe_only`
- 对 Agent 回合设置更短超时
- 允许 Agent 只在高价值回合触发

### 风险 2：输出不可控

应对：

- 强制结构化输出
- 强制 schema 校验
- 强制规则层最终裁决

### 风险 3：`tauri_commands.rs` 进一步膨胀

应对：

- 新逻辑放入 `agent_runtime.rs`
- `tauri_commands.rs` 只保留参数校验与 orchestrator 调用

### 风险 4：调试困难

应对：

- V1 必做 Trace
- 前端提供 Agent 调试页签
- 所有回退都保留明确理由

## 16. 推荐的最小可交付范围

第一版不要贪多，建议只交付：

- 单个 `DirectorAgent`
- 单个接入点：`execute_player_action`
- 单个运行模式：`observe_only`
- 单个 Trace 面板
- 单个候选工具：`GeneratePlotCandidate`

这样做的好处是：

- 不会破坏当前可玩链路
- 可以真实观察 Agent 对剧情质量的提升空间
- 可以先建立日志、护栏、测试基线

## 17. 结论

`Nobody` 适合接入 Agent，但最优路径不是“让 Agent 接管游戏”，而是“让 Agent 成为受约束的编排层”。

V1 的推荐方向是：

- 先做单 Agent
- 先做观察模式
- 先做结构化提案
- 先做护栏与回退

等 V1 跑稳后，再进入 NPC Agent、多 Agent 协作、世界长期演化等更高阶能力。
