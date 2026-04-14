# NoName Agent T1 文件级实现清单

更新时间: 2026-04-08
状态: 可直接开工
范围: `T1 Core 类型与配置骨架`
关联文档:
- `noname-agent-v1.md`
- `noname-v1-blueprint.md`
- `noname-v1-task-list.md`

## 1. 目标

这份文档只服务一件事:

把 `T1` 从“任务包”继续下钻到“文件级实现清单”，明确:

- 每个文件负责什么
- 先放哪些类型
- 暂时不要做什么
- 每个文件做到什么程度算完成

`T1` 的目标不是让 `NoName Agent` 真的跑起来，而是为后续 `T2 / T3 / T4 / T5` 提供稳定的类型、配置和错误底座。

## 2. T1 范围边界

### 本阶段要完成

- 建立 `NoName` 命名空间下的核心枚举和基础对象
- 建立统一配置对象与默认配置
- 建立统一错误对象与错误分类
- 在 `lib.rs` 中注册新模块
- 为核心对象补基础单测

### 本阶段不要提前做

- 不实现真正的 runtime
- 不实现 graph executor
- 不接入 tauri command
- 不接入 LLM 调度
- 不接入 guardrail 校验
- 不接入 memory/context 检索

一句话说，`T1` 只做“骨架底板”，不做“运行逻辑”。

## 3. 文件清单

`T1` 建议只动 4 个文件:

1. `src-tauri/src/noname_types.rs`
2. `src-tauri/src/noname_config.rs`
3. `src-tauri/src/noname_errors.rs`
4. `src-tauri/src/lib.rs`

如果需要单测，优先放在各自文件的 `#[cfg(test)] mod tests` 中，不建议在 `tests.rs` 里提前集中化。

## 4. 文件级实现要求

## 4.1 `src-tauri/src/noname_types.rs`

### 文件职责

承载 `NoName Agent` 的最小共享对象模型。

这个文件里的类型应该满足两个要求:

- 后续所有 `noname_*` 模块都能直接复用
- 即使 runtime 还没写，也可以独立序列化、构造、测试

### 第一批建议放入的类型

#### 枚举

- `NoNameMode`
  - `Disabled`
  - `ObserveOnly`
  - `Assisted`

- `NoNameRole`
  - `Director`
  - `WorldCurator`
  - `NpcIntent`
  - `CombatNarrator`
  - `System`

- `NoNameProposalKind`
  - `PlotCandidate`
  - `WorldPatchProposal`
  - `NpcIntentProposal`
  - `CombatNarration`
  - `Diagnostic`

- `NoNameEnvelopeKind`
  - `Task`
  - `Proposal`
  - `CapabilityCall`
  - `CapabilityResult`
  - `TraceEvent`
  - `Diagnostic`

- `NoNameTraceStage`
  - `CollectTurnInput`
  - `BuildContextBundle`
  - `PlanTurn`
  - `ExecuteToolSteps`
  - `AssembleProposal`
  - `ValidateProposal`
  - `PersistTrace`
  - `Fallback`

#### 基础结构体

- `NoNameIdentity`
  - `trace_id`
  - `session_id`
  - `turn_id`
  - `role`

- `NoNameMeta`
  - `created_at_ms`
  - `labels`
  - `token_budget`
  - `timeout_ms`

- `NoNameEnvelope`
  - `identity`
  - `kind`
  - `payload`
  - `meta`

- `NoNameProposalRef`
  - `proposal_id`
  - `kind`
  - `producer_role`

### `payload` 的第一阶段建议

`T1` 不必急着把 `payload` 强类型化成完整协议对象。

建议第一版先用:

- `serde_json::Value`

这样可以先稳定 envelope 边界，后续在 `T3` 再逐步替换成协议层对象。

### 需要的辅助实现

- `impl Default for NoNameMode`
  - 默认值建议为 `Disabled`
- `impl NoNameMode`
  - `is_enabled()`
  - `allows_apply()`
- `impl NoNameEnvelope`
  - 最小 `new()` 构造函数
- `#[serde(rename_all = "camelCase")]`
  - 与现有 `context_builder.rs`、`memory_layers.rs` 风格保持一致

### 暂时不要放进来的东西

- 真正的 `NoNameTrace`
- 真正的 `NoNameTaskLifecycle`
- capability descriptor
- context packet
- guardrail result

这些属于 `T2/T3/T4/T5`。

### 完成标准

- 所有类型都可 `Serialize + Deserialize + Clone + Debug`
- 基础枚举可以稳定 round-trip
- `NoNameEnvelope::new()` 能在单测里直接构造出对象

### 建议单测

- `mode_defaults_to_disabled`
- `observe_only_is_enabled_but_not_applyable`
- `envelope_round_trip_serialization`
- `proposal_ref_keeps_kind_and_role`

## 4.2 `src-tauri/src/noname_config.rs`

### 文件职责

定义 `NoName Agent` 的运行配置、预算策略和默认值。

这个文件应当回答两个问题:

- 当前回合允许 Agent 做到什么程度
- 当前运行时能消耗多少预算和保留多少调试信息

### 第一批建议放入的类型

- `NoNameTokenBudget`
  - `total`
  - `context_reserved`
  - `planning_reserved`
  - `tool_reserved`
  - `response_reserved`

- `NoNameTimeoutPolicy`
  - `planning_timeout_ms`
  - `tool_timeout_ms`
  - `total_turn_timeout_ms`

- `NoNameCapabilityPolicy`
  - `whitelist`
  - `allow_prompt_capabilities`
  - `allow_resource_capabilities`
  - `allow_tool_capabilities`

- `NoNameTracePolicy`
  - `enabled`
  - `max_recent_traces`
  - `include_payload_preview`

- `NoNameConfig`
  - `mode`
  - `token_budget`
  - `timeout_policy`
  - `capability_policy`
  - `trace_policy`

### 设计建议

#### 默认配置

建议提供:

- `impl Default for NoNameConfig`
- 默认值以 `ObserveOnly` 为目标形态设计，但 `mode` 仍可先默认为 `Disabled`

也就是:

- 数据结构上准备好 `observe_only`
- 仓库运行时默认仍然不开启

#### 预设构造器

建议至少提供:

- `NoNameConfig::disabled()`
- `NoNameConfig::observe_only()`
- `NoNameConfig::assisted()`

这样后续 `tauri_commands.rs` 接 runtime 时不需要在命令层手写策略拼装。

#### 预算一致性校验

建议加一个轻量方法:

- `validate()`

检查:

- 各类 token reserve 之和不能明显超过 total
- timeout 不能为 0
- trace retention 不能为负值风格异常

`T1` 里不必引入复杂校验框架，返回 `Result<(), NoNameConfigError>` 即可。

### 暂时不要放进来的东西

- 环境变量加载
- 配置文件持久化
- 前端设置同步
- 动态热更新

### 完成标准

- 可以一行构造 `NoNameConfig::observe_only()`
- `validate()` 能识别明显错误配置
- 所有配置对象都能序列化/反序列化

### 建议单测

- `default_config_is_disabled`
- `observe_only_preset_enables_trace`
- `invalid_budget_fails_validation`
- `invalid_timeout_fails_validation`

## 4.3 `src-tauri/src/noname_errors.rs`

### 文件职责

建立 `NoName Agent` 的统一错误分类和公共错误出口。

这里的设计目标不是“错误非常复杂”，而是“后续所有层都能挂到同一棵错误树上”。

### 第一批建议放入的类型

#### 顶层分类

- `NoNameErrorKind`
  - `Config`
  - `Protocol`
  - `Capability`
  - `Memory`
  - `Context`
  - `Guardrail`
  - `Runtime`
  - `Trace`
  - `Unknown`

#### 顶层错误对象

- `NoNameError`
  - `kind`
  - `message`
  - `code`
  - `recoverable`

#### 子域错误

- `NoNameConfigError`
- `NoNameProtocolError`
- `NoNameCapabilityError`
- `NoNameMemoryError`
- `NoNameContextError`
- `NoNameGuardrailError`

### 设计建议

#### 顶层 + 子域双层结构

建议不要一开始就把所有错误都塞进一个超大枚举。

更稳的做法是:

- 子域错误负责表达局部语义
- `NoNameError` 负责跨模块统一对外输出

#### 与现有仓库风格对齐

当前仓库已有 [app_error.rs](D:/Nobody/src-tauri/src/app_error.rs) 这种轻量错误结构，所以 `NoNameError` 建议保持相近风格:

- 可序列化
- 有 `kind`
- 有 `message`
- 支持 `Display`
- 支持 `Error`

#### 建议提供的转换

至少实现:

- `From<NoNameConfigError> for NoNameError`
- `From<NoNameProtocolError> for NoNameError`
- `From<NoNameCapabilityError> for NoNameError`
- `From<NoNameMemoryError> for NoNameError`
- `From<NoNameContextError> for NoNameError`
- `From<NoNameGuardrailError> for NoNameError`
- `From<NoNameError> for AppError`

这样后面接 tauri command 时，可以自然回到现有错误出口。

### 暂时不要放进来的东西

- thiserror 宏重构全仓库
- anyhow 到处透传
- 复杂 source-chain

`T1` 要的是稳定、轻量、可序列化，不是错误系统大改造。

### 完成标准

- 所有错误都可统一转成 `NoNameError`
- `NoNameError` 可转成现有 `AppError`
- 单测里可以验证 kind 和 recoverable 标记

### 建议单测

- `config_error_maps_to_top_level_error`
- `protocol_error_is_recoverable`
- `guardrail_error_can_convert_to_app_error`

## 4.4 `src-tauri/src/lib.rs`

### 文件职责

只做模块注册，不在 `T1` 提前接入 runtime 或 tauri 命令。

### 本阶段需要修改的内容

增加模块导出:

- `pub mod noname_types;`
- `pub mod noname_config;`
- `pub mod noname_errors;`

### 本阶段不要改的内容

- 不加 `manage()` 状态
- 不加 invoke handler
- 不改 `run()` 主流程

### 完成标准

- 新模块可以被其他 `src-tauri/src/*.rs` 正常引用
- `cargo test` 至少能编译通过新增模块相关部分

## 5. 推荐实现顺序

建议严格按下面顺序做，能减少来回返工:

1. 先写 `noname_types.rs`
2. 再写 `noname_errors.rs`
3. 再写 `noname_config.rs`
4. 最后改 `lib.rs`
5. 跑新增单测与最小编译验证

这样安排的原因是:

- `config` 依赖 `mode`
- `config.validate()` 依赖错误类型
- `lib.rs` 最后改，便于在文件尚未稳定时减少编译噪音

## 6. 建议提交切片

如果按小提交推进，建议切成 3 个 commit:

1. `docs: add noname t1 file checklist`
2. `feat(noname): add core types and errors skeleton`
3. `feat(noname): add config skeleton and register modules`

## 7. 完成定义

`T1` 完成，不等于 Agent 已可运行。

`T1` 完成的标志是:

- 后续所有 `noname_*` 文件终于有稳定依赖的公共类型
- runtime/trace/protocol/memory 不需要再重新定义 mode、envelope、基础错误
- 可以开始进入 `T2 Trace 与图执行骨架`

## 8. 下一步衔接

`T1` 完成后，下一步直接进入:

- `T2 Trace 与图执行骨架`

届时建议优先新增:

- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_graph.rs`
- `src-tauri/src/noname_runtime.rs`

并复用本阶段已经落好的:

- `NoNameMode`
- `NoNameEnvelope`
- `NoNameTraceStage`
- `NoNameConfig`
- `NoNameError`
