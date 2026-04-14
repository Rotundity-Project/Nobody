# NoName Agent V1 实施蓝图

更新时间: 2026-04-09
状态: V1 已完成，`assisted skeleton` 进行中
目标: 将 `NoName Agent` 从设计文档推进到可运行的 V1 骨架，并为后续受控应用阶段建立基线
关联文档:
- `noname-agent-v1.md`
- `noname-memory-context-v1.md`
- `noname-framework-protocol-v1.md`
- `domain-model-v2.md`

## 1. 蓝图目标

V1 不追求完整多 Agent 生态，而是交付一个可以在 `Nobody` 中实际运行、可调试、可回退的最小版本。

V1 的核心目标：

- 在后端建立 `NoName Runtime` 骨架
- 在主剧情推进链路中接入 `observe_only` 模式
- 建立统一的类型、配置、错误与 Trace 体系
- 建立最小可用的记忆写入与上下文构建能力
- 建立本地版 `NNCP-T / NNCP-A` 基础对象

## 1.1 当前状态快照

截至 `2026-04-09`，当前代码状态为：

- `V1` 基础闭环已经完成
- `DirectorAgent` 已接入 `execute_player_action`
- 结构化 `NoNameProposal` 已落地
- Guardrail 已接入并可产出 `accept / repair / reject`
- 前端已经能查看 trace、proposal、guardrail、fallback 调试摘要
- 当前正在推进 `assisted skeleton`
- 当前尚未进入“真正应用 proposal 到主剧情结果”的阶段

## 2. V1 范围边界

### 包含

- 单个主角色：`DirectorAgent`
- 单个主接入点：`execute_player_action`
- 单机本地运行
- 本地 Capability Registry
- Working / Episodic / Semantic / Narrative 的最小记忆骨架
- GSSC 风格上下文构建流程
- Trace 记录与调试读取命令

### 不包含

- 真正的远程 MCP/A2A/ANP 网络实现
- 完整多 Agent 自由协作
- 外部向量数据库和图数据库
- 前端复杂可视化 Agent 控制台
- 直接由 Agent 接管主剧情最终落地

## 3. 实施原则

- 先骨架，后智能。
- 先 observe_only，后 assisted。
- 先本地协议对象，后真实网络协议。
- 先统一 Envelope / Trace / Config，后增加能力。
- 任何阶段都不允许破坏现有经典链路。

## 4. 阶段拆解

## 阶段 0：仓库与文档收束 `已完成`

目标：让实现工作有稳定文档基线和清晰落点。

任务：

- 建立 `docs/README.md` 与 `docs/architecture/README.md`
- 明确 `docs/` 与 `.kiro/` 的职责边界
- 产出本蓝图文档
- 保持 `NoName Agent` 三份专题文档为基线输入

完成标准：

- 团队可以清楚知道“正式设计看哪里、草稿看哪里”
- Agent 文档不再分散难找

## 阶段 1：Core Skeleton `已完成`

目标：建立 NoName Agent 的最小运行时核心。

建议新增文件：

- `src-tauri/src/noname_types.rs`
- `src-tauri/src/noname_config.rs`
- `src-tauri/src/noname_errors.rs`
- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_graph.rs`

任务：

- 定义核心类型：mode、trace、task、envelope、proposal
- 定义统一错误体系
- 定义运行模式配置
- 实现最小图执行器
- 提供 trace 存取接口

完成标准：

- 可以构造一次空的 Agent 回合
- 可以记录 trace
- 不影响现有游戏逻辑

## 阶段 2：Protocol & Capability Skeleton `已完成`

目标：把工具、资源、提示模板纳入统一能力抽象。

建议新增文件：

- `src-tauri/src/noname_protocol_types.rs`
- `src-tauri/src/noname_protocol_tool.rs`
- `src-tauri/src/noname_protocol_agent.rs`
- `src-tauri/src/noname_capability_base.rs`
- `src-tauri/src/noname_capability_registry.rs`
- `src-tauri/src/noname_resources.rs`
- `src-tauri/src/noname_prompt_catalog.rs`

任务：

- 定义 `NNCP-T / NNCP-A` 的对象模型
- 定义 Capability Descriptor
- 建立本地 registry
- 将工具、资源、prompt 统一纳管

完成标准：

- 能列出本地 capability
- 能统一调用本地 tool / resource / prompt
- 协议对象能写入 trace

## 阶段 3：Memory & Context Skeleton `已完成`

目标：补齐 NoName Agent 的最小记忆和上下文工程。

建议新增文件：

- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_memory_store.rs`
- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_note_store.rs`

任务：

- 复用现有 `memory_layers.rs`，补 NarrativeMemory
- 建 WorkingMemory / EpisodicMemory 的基本写入逻辑
- 建 NoNameContextBuilder
- 实现 `Gather -> Score -> Select -> Structure -> Compress` 的最小版

完成标准：

- 单回合可以产出结构化 Context Packet
- 可按 token budget 裁剪
- 可区分 DirectorAgent 的专属上下文

## 阶段 4：DirectorAgent 接入 `已完成`

目标：让主剧情推进链路进入 `observe_only` Agent 模式。

建议新增文件：

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_prompts.rs`
- `src-tauri/src/noname_tools.rs`

建议修改文件：

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/lib.rs`

任务：

- 定义 `DirectorAgent`
- 定义最小 prompt 模板
- 提供 `GeneratePlotCandidate` capability
- 在 `execute_player_action` 中接入 observe_only 分支
- 将 Agent 结果写入 trace，但不参与最终落地

完成标准：

- 一次玩家行动可以触发 Agent 规划
- Agent 结果可被查看
- 经典链路结果保持不变

## 阶段 5：Guardrail Gateway 接入 `已完成`

目标：让 Agent 提案进入校验体系，准备 `assisted` 模式。

建议新增文件：

- `src-tauri/src/noname_guardrails.rs`

任务：

- 统一接入 `plot_consistency.rs`
- 统一接入 `state_patch_validator.rs`
- 统一接入 `entity_validator.rs`
- 统一接入 `numeric_guard.rs`
- 输出 accept / reject / repair 结果

完成标准：

- Agent proposal 可被校验并给出明确拒绝原因
- 失败时可 fallback

## 阶段 6：前端调试接入 `已完成`

目标：让 Agent 调试信息可见，但不重做 UI。

建议修改文件：

- `src/types/game.ts`
- `src/stores/gameStore.ts`
- `src/components/GameInfoCenterDialog.vue`
- 可新增 `src/components/AgentTracePanel.vue`

任务：

- 暴露运行模式与 trace 读取
- 在开发模式显示 Agent Trace
- 展示上下文来源、capability 调用、guardrail 结果

完成标准：

- 前端可查看最近 trace
- 可快速判断 Agent 是否有价值

## 阶段 7：Assisted Skeleton `进行中`

目标：在不改变经典主链路优先级的前提下，为 proposal 进入辅助应用分支建立最小能力。

当前已完成：

- `NoNameMode::Assisted` 配置预设
- `get_noname_mode / set_noname_mode` 运行时入口
- `DirectorAgent` 输出结构化 `NoNameProposal`
- runtime 可将 proposal 标记为 `applyable`
- trace 可记录 proposal、guardrail、fallback、阶段跳转
- 前端调试信息可显示 proposal 是否 `ready`

当前未完成：

- proposal 还没有真正影响最终剧情文本或状态落地
- 还没有“应用 proposal 后的二次 guardrail + 回退”链路
- 还没有完整的 `disabled / observe_only / assisted` 集成测试矩阵

阶段完成标准：

- `assisted` 模式下 proposal 可以进入受控应用预备分支
- 只有通过 guardrail 的 proposal 才允许进入下一阶段
- 任意失败都可以回退到经典链路

## 5. 文件落点图

### 正式文档

- `docs/architecture/noname-agent-v1.md`
- `docs/architecture/noname-memory-context-v1.md`
- `docs/architecture/noname-framework-protocol-v1.md`
- `docs/architecture/noname-v1-blueprint.md`

### 后端主实现

- `src-tauri/src/noname_*.rs`

### 前端调试接入

- `src/components/*`
- `src/stores/gameStore.ts`
- `src/types/game.ts`

### 本地规划与交接

- `.kiro/specs/Nobody/current/`
- `.kiro/specs/Nobody/handoffs/`

## 6. 建议的首批类型对象

首批必须有：

- `NoNameMode`
- `NoNameTrace`
- `NoNameEnvelope`
- `NoNameCapabilityDescriptor`
- `NoNameTask`
- `NoNameContextPacket`
- `NoNameProposal`
- `NoNameGuardrailResult`

当前状态：上述对象已具备，且 `NoNameProposal` 已进入 runtime / trace / 前端调试链路。

## 7. 测试策略

### 单元测试

- 类型对象序列化/反序列化
- Graph 状态跳转
- Capability Registry 注册与发现
- Context Builder 的 score/select 结果
- Guardrail accept / reject 分支

### 集成测试

- `execute_player_action` 在 `disabled` 模式下行为不变
- `observe_only` 模式会产出 trace
- Agent 失败时 fallback 到经典链路
- `assisted` 模式下 proposal 仅在 guardrail 通过时进入 `ready` 状态

### 属性测试

- Agent 输出不会破坏剧情状态机
- Agent 输出不会破坏地图位置一致性
- Agent 输出不会破坏选项数量边界

## 8. 风险与控制

### 风险 1：过早做太多模块

控制：

- V1 只做骨架，不做完整多 Agent 网络

### 风险 2：`tauri_commands.rs` 继续膨胀

控制：

- 所有 NoName 新逻辑优先写入 `noname_*.rs`
- `tauri_commands.rs` 只做入口编排

### 风险 3：记忆层过重

控制：

- V1 只用 SQLite + 内存
- 不急着接入外部向量库

### 风险 4：调试困难

控制：

- Trace 先于智能能力落地
- 协议对象统一使用 Envelope

### 风险 5：过早让 Agent 直接改主结果

控制：

- 当前仅推进 `assisted skeleton`
- 真实应用 proposal 前必须先定义受控边界与二次回退链路

## 9. 文档使用说明

如果要推进实现，建议按这个顺序阅读：

1. `noname-agent-v1.md`
2. `noname-memory-context-v1.md`
3. `noname-framework-protocol-v1.md`
4. `noname-v1-blueprint.md`

如果要开始写代码，建议按这个顺序落文件：

1. `noname_types.rs`
2. `noname_config.rs`
3. `noname_errors.rs`
4. `noname_trace.rs`
5. `noname_runtime.rs`
6. `noname_protocol_types.rs`
7. `noname_capability_registry.rs`
8. `noname_context_builder.rs`

当前补充：如果要继续推进下一阶段，优先阅读 `noname_runtime.rs`、`noname_guardrails.rs` 与 `tauri_commands.rs` 中的 `assisted` 相关链路。

## 10. 结论

`NoName Agent V1` 当前不再是“待实现设计”，而是：

- 有骨架
- 有协议
- 有记忆
- 有上下文
- 有 trace
- 有回退
- 有 proposal
- 有 guardrail

因此，`V1` 可以视为已完成。真正的下一阶段目标，不是再重复搭框架，而是把 `assisted skeleton` 推进成“受控应用 proposal”的稳定实现。
