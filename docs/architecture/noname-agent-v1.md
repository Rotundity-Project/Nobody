# NoName Agent V1 设计文档

更新时间: 2026-04-07
状态: 设计基线
适用项目: `Nobody`
关联文档:
- `docs/ARCHITECTURE.md`
- `docs/architecture/domain-model-v2.md`
- `docs/architecture/agent-framework-v1.md`

## 1. 定义

`NoName Agent` 是 `Nobody` 项目的自定义 Agent 框架名称。

它不是直接照搬某一个通用 Agent 框架，而是面向 `Nobody` 的游戏叙事、世界状态、一致性校验和长期演化需求，做的一套“受约束的 Agent Runtime”。

一句话定义：

`NoName Agent = 状态图驱动 + 消息协议驱动 + 角色分工驱动 + 强护栏落地的 Nobody 原生 Agent 框架`

## 2. 为什么不直接选单一框架

`Nobody` 的核心不是“多智能体多聊几轮”，而是：

- 世界状态必须可追踪
- 剧情推进必须可回放
- 数值、地图、角色、章节必须受规则约束
- Agent 输出必须能被校验和拒绝
- 失败时必须有稳定降级路径

这决定了 `Nobody` 不能简单照搬通用框架默认范式。

## 3. NoName Agent 的来源思想

NoName Agent 建议吸收四类框架思想，但只取最适合 `Nobody` 的部分。

### 3.1 LangGraph: 作为流程主骨架

吸收点：

- 状态图
- 节点与边
- 条件分支
- 回退与循环
- 长时运行与可追踪执行

在 NoName Agent 中对应为：

- 一回合推进流程图
- Agent 节点、工具节点、校验节点、降级节点
- `observe_only` / `assisted` 模式切换

### 3.2 AgentScope: 作为消息与工程编排模型

吸收点：

- Message 作为基础对象
- State 作为一等公民
- Pipeline / MsgHub 的工程组织方式

在 NoName Agent 中对应为：

- `NoNameMessage`
- `NoNameEnvelope`
- `NoNameContextBundle`
- `NoNamePipelineStep`

### 3.3 CAMEL: 作为角色体系来源

吸收点：

- 角色分工
- 协作责任边界
- 面向长期任务的 agent workforce 思维

在 NoName Agent 中对应为：

- `DirectorAgent`
- `NpcIntentAgent`
- `WorldCuratorAgent`
- `CombatNarratorAgent`

### 3.4 AutoGen: 作为局部协作模式来源

吸收点：

- 对话式协作
- 多角色讨论后产出结果
- 人工介入与调试友好性

在 NoName Agent 中对应为：

- 特殊场景下的双 Agent 协作
- 离线生成任务
- 调试型 agent 评审模式

注意：

AutoGen 思路不作为 NoName Agent 的主循环骨架，只作为局部能力扩展。

## 4. 核心定位

NoName Agent 的定位不是“取代引擎”，而是“增强引擎”。

职责分层如下：

- Agent 负责：理解目标、规划步骤、调用工具、产出提案
- 引擎负责：校验提案、计算规则、提交状态、记录结果

这意味着：

- Agent 没有最终裁决权
- Agent 没有原始状态直写权
- 引擎仍然是单一事实源的守门人

## 5. 设计原则

### 5.1 Rule-first

一切 Agent 结果都必须经过规则层。

### 5.2 State-first

Agent 不是面对一段 prompt 工作，而是面对结构化状态工作。

### 5.3 Trace-first

所有 Agent 回合都必须可追踪、可诊断、可复盘。

### 5.4 Proposal-only

Agent 只能提出候选结果，不能直接落地最终状态。

### 5.5 Fallback-always

任何 Agent 失败都必须能切回现有链路。

## 6. NoName Agent 总体架构

建议分为 7 层。

### 6.1 Presentation Layer

位置：`src/`

职责：

- 展示 Agent 诊断信息
- 暴露运行模式开关
- 在开发模式下展示 trace

### 6.2 Command Orchestration Layer

位置：`src-tauri/src/tauri_commands.rs`

职责：

- 收集本回合输入
- 调用 `NoNameRuntime`
- 处理成功、拒绝、降级分支

### 6.3 NoName Runtime Layer

建议新增：`src-tauri/src/noname_runtime.rs`

职责：

- 驱动状态图
- 调度角色 Agent
- 执行工具节点
- 聚合 proposal
- 调用 guardrail gateway

### 6.4 Agent Role Layer

建议新增：

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_prompts.rs`

职责：

- 定义角色身份
- 定义每个角色的输入输出约束
- 定义提示模板与行为限制

### 6.5 Tool Layer

建议新增：`src-tauri/src/noname_tools.rs`

职责：

- 暴露受控工具
- 连接现有 memory / plot / world / entity 能力

### 6.6 Guardrail Layer

复用已有模块：

- `plot_consistency.rs`
- `state_patch_validator.rs`
- `entity_validator.rs`
- `numeric_guard.rs`

职责：

- 校验 proposal
- 判断是否可接受
- 输出拒绝理由

### 6.7 Persistence and Trace Layer

建议新增：`src-tauri/src/noname_trace.rs`

职责：

- 保存每回合 trace
- 提供调试读取接口
- 支持回放和比对

## 7. 推荐模块命名

建议用 `noname_*.rs` 作为统一前缀，避免和通用 `agent_*` 命名混淆。

建议新增文件：

- `src-tauri/src/noname_types.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_tools.rs`
- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_prompts.rs`
- `src-tauri/src/noname_graph.rs`
- `src-tauri/src/noname_guardrails.rs`

## 8. 状态图模型

NoName Agent 的主循环建议使用显式状态图。

### 8.1 回合主图

建议节点：

1. `CollectTurnInput`
2. `BuildContextBundle`
3. `PlanTurn`
4. `ExecuteToolSteps`
5. `AssembleProposal`
6. `ValidateProposal`
7. `ApplyAcceptedProposal`
8. `FallbackToClassicFlow`
9. `PersistTrace`

### 8.2 边的定义

- `CollectTurnInput -> BuildContextBundle`
- `BuildContextBundle -> PlanTurn`
- `PlanTurn -> ExecuteToolSteps`
- `ExecuteToolSteps -> AssembleProposal`
- `AssembleProposal -> ValidateProposal`
- `ValidateProposal -> ApplyAcceptedProposal` 当校验通过
- `ValidateProposal -> FallbackToClassicFlow` 当校验失败
- `ApplyAcceptedProposal -> PersistTrace`
- `FallbackToClassicFlow -> PersistTrace`

### 8.3 为什么必须显式状态图

因为 `Nobody` 已经有：

- 章节生命周期
- 交互状态机
- 快速模式补全
- 世界事实写回
- 多类校验与回退

这些都天然适合状态图，不适合隐式堆在一个大函数里。

## 9. 角色体系

V1 不建议一开始上多 Agent 自由协作，但建议先把角色体系定义好。

### 9.1 DirectorAgent

职责：

- 决定本回合剧情目标
- 判断节奏阶段
- 选择工具调用顺序
- 输出候选剧情计划

权限：

- 可读剧情、角色、世界摘要
- 可调用候选剧情生成工具
- 不可写最终状态

### 9.2 WorldCuratorAgent

职责：

- 识别应沉淀的世界事实
- 提出世界事实 patch
- 帮助维护长期叙事一致性

权限：

- 可读事件和章节摘要
- 可提世界事实候选 patch
- 不可绕过世界校验直接落库

### 9.3 NpcIntentAgent

职责：

- 生成关键 NPC 的短期意图
- 维持 NPC 行为连续性

权限：

- 可读 NPC 档案、关系、地理位置
- 可提出 NPC 意图，不可直接改世界状态

### 9.4 CombatNarratorAgent

职责：

- 在已有战斗裁决结果之上生成解释与叙事包装

权限：

- 不参与战斗结果裁决
- 只负责解释层文本与影响摘要

## 10. NoName 消息协议

建议 NoName Agent 统一使用消息信封，而不是直接传裸字符串。

### 10.1 基础结构

```ts
type NoNameEnvelope<T = Record<string, unknown>> = {
  envelopeId: string
  turnId: string
  source: "player" | "director" | "tool" | "guardrail" | "system"
  role: string
  kind: string
  content: string
  payload: T
  meta: {
    chapterIndex?: number
    location?: string
    traceTag?: string
    createdAt: number
  }
}
```

### 10.2 设计收益

- 文本内容和结构化数据分离
- 更易做日志与回放
- 更易插入校验和调试
- 更适合多角色扩展

## 11. Context Bundle 设计

建议把 Agent 输入上下文统一封装成 `NoNameContextBundle`。

```ts
type NoNameContextBundle = {
  turn: {
    turnId: string
    actionType: string
    actionText: string
  }
  player: {
    name: string
    realm: string
    location: string
    combatStatus?: Record<string, unknown>
  }
  plot: {
    interactionState: string
    chapterIndex: number
    chapterTitle: string
    recentParagraphs: string[]
  }
  world: {
    facts: Record<string, unknown>[]
    reachableLocations: string[]
    importantNpcs: Record<string, unknown>[]
  }
  diagnostics: {
    quickModeAllowed: boolean
    strictMode: boolean
  }
}
```

这个对象应当由后端构建，不应交给前端拼装。

## 12. Tool 设计

V1 建议只暴露白名单工具。

### 12.1 读取类工具

- `ReadPlotSnapshot`
- `ReadRecentEvents`
- `ReadWorldFacts`
- `ReadCharacterProfile`
- `ReadReachableLocations`

### 12.2 生成类工具

- `GeneratePlotCandidate`
- `GenerateOptionHints`
- `GenerateSceneConflictHints`

### 12.3 提案类工具

- `ProposeWorldFactPatch`
- `ProposeNpcIntent`

### 12.4 禁止直接暴露的能力

- 直接写 `GameState`
- 直接写 `PlotState`
- 直接写 `WorldRegistry`
- 绕过现有 validator 的 patch 提交

## 13. Guardrail Gateway

NoName Agent 必须有统一的 Guardrail Gateway。

建议新增统一结果结构：

```ts
type NoNameGuardrailResult = {
  accepted: boolean
  severity: "info" | "warning" | "critical"
  reasons: string[]
  repairedPayload?: Record<string, unknown>
}
```

建议流程：

1. Proposal schema 校验
2. 剧情一致性校验
3. 状态补丁校验
4. 实体校验
5. 数值校验
6. 最终 accept / reject

## 14. 运行模式

建议 NoName Agent 支持三种模式。

### 14.1 `disabled`

- 完全关闭
- 走现有引擎链路

### 14.2 `observe_only`

- 跑完整 Agent 流程
- 记录 trace
- 不采用最终结果

适合作为第一阶段默认模式。

### 14.3 `assisted`

- Agent proposal 可被采用
- 但必须通过全部 guardrails

## 15. Nobody 中的接入点

### 15.1 第一接入点

首选接入：`execute_player_action`

原因：

- 主剧情循环价值最高
- 已有完整状态和诊断链路
- 最适合观察 Agent 对叙事质量的实际提升

### 15.2 第二接入点

次级接入：`initialize_plot`

适合做：

- 开场章节规划
- 第一回合世界补锚

### 15.3 第三接入点

后续可接：

- `travel_to_location`
- 战斗解释链路
- 世界事实补全链路

## 16. Nobody 中的文件映射建议

### 16.1 后端新增

- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_graph.rs`
- `src-tauri/src/noname_types.rs`
- `src-tauri/src/noname_tools.rs`
- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_prompts.rs`
- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_guardrails.rs`

### 16.2 后端修改

- `src-tauri/src/lib.rs`
- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/context_builder.rs`

### 16.3 前端修改

- `src/stores/gameStore.ts`
- `src/types/game.ts`
- `src/components/GameInfoCenterDialog.vue`
- 可能新增 `src/components/AgentTracePanel.vue`

## 17. 追踪与调试

NoName Agent 必须把 Trace 当成框架内建能力。

建议结构：

```ts
type NoNameTrace = {
  traceId: string
  turnId: string
  mode: string
  graphPath: string[]
  roleCalls: Array<{
    role: string
    summary: string
  }>
  toolCalls: Array<{
    tool: string
    ok: boolean
    summary: string
  }>
  guardrail: {
    accepted: boolean
    reasons: string[]
  }
  fallbackUsed: boolean
  elapsedMs: number
}
```

前端只需要先支持：

- 最近 10 条 trace 查看
- 开发模式下展示
- 允许复制 diagnostics

## 18. V1 最小可交付范围

不要一开始就做成复杂多 Agent 系统。

V1 建议最小范围：

- 单个 `DirectorAgent`
- 单个状态图主循环
- 单个提案类型 `PlotCandidateProposal`
- 单个接入点 `execute_player_action`
- 单个 trace 读取命令
- 默认 `observe_only`

## 19. 实施路线

### Phase 0: 基础准备

- 修复现有 fallback 与测试失真
- 统一主链路 diagnostics 字段
- 整理 `tauri_commands.rs` 中上下文构建逻辑

### Phase 1: Runtime 骨架

- 新增 `noname_types.rs`
- 新增 `noname_trace.rs`
- 新增 `noname_runtime.rs`
- 新增运行模式配置

### Phase 2: 单 Agent 接入

- 新增 `DirectorAgent`
- 新增 `GeneratePlotCandidate`
- 在 `execute_player_action` 中接入 `observe_only`

### Phase 3: Guardrail 集成

- 接入现有 consistency / patch / entity / numeric 校验
- 支持 reject 和 fallback

### Phase 4: 前端调试面板

- 增加 `Agent Trace` 页签
- 展示最近 trace

### Phase 5: 世界与 NPC 扩展

- `WorldCuratorAgent`
- `NpcIntentAgent`

## 20. 测试建议

### 20.1 单元测试

- ContextBundle 构建
- Graph 节点跳转
- Tool 白名单
- Guardrail 拒绝逻辑

### 20.2 集成测试

- `disabled` 模式不改变原有行为
- `observe_only` 只记录不落地
- `assisted` 在校验通过时采用结果
- Agent 失败后自动 fallback

### 20.3 属性测试

- Agent 输出不破坏章节状态机
- Agent 输出不破坏地图位置一致性
- Agent 输出不破坏选项数范围

## 21. 这套框架的独特性

NoName Agent 的独特之处不在“比通用框架更大”，而在“更适合 Nobody”。

它的独特点在于：

- 以游戏引擎为中心，不以聊天循环为中心
- 以状态图为骨架，不以自由对话为骨架
- 以 proposal + guardrail 为核心，不以 agent 直写为核心
- 以世界一致性和长期可回放为第一目标

## 22. 命名建议

正式名称：`NoName Agent`

子系统命名建议：

- `NoName Runtime`
- `NoName Graph`
- `NoName Trace`
- `NoName Guardrail Gateway`
- `NoName Director`

## 23. 结论

对 `Nobody` 来说，最适合的路线不是“引入某个现成框架并围绕它重构”，而是：

- 用 `LangGraph` 思想做主骨架
- 用 `AgentScope` 思想做消息与工程组织
- 用 `CAMEL` 思想做角色分工
- 用 `AutoGen` 思想做局部协作扩展
- 最终形成 `NoName Agent` 这套 `Nobody` 原生框架

这会比单纯接一个外部框架更稳，也更适合后续长期演进。
