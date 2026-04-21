# NoName Agent V1 开发任务清单

更新时间: 2026-04-17
状态: V1 基础闭环已完成，已进入 `T7 assisted skeleton / 受控应用 proposal` 阶段
目标: 将 `NoName Agent` V1 设计拆解为可按顺序实施的开发任务
关联文档:
- `noname-agent-v1.md`
- `noname-memory-context-v1.md`
- `noname-framework-protocol-v1.md`
- `noname-v1-blueprint.md`
- `noname-t1-file-checklist.md`
- `noname-t7-file-checklist.md`

## 1. 使用说明

这份清单面向开发执行，不再重复解释总体设计理念，而是回答：

- 先做什么
- 后做什么
- 每一步改哪些文件
- 每一步完成后如何判断“算完成”

建议按顺序推进，不要跳阶段同时大规模并行实现。

## 1.1 当前实现状态

截至 `2026-04-17`，当前仓库中的 `NoName Agent` 实现状态如下：

- `T0` 到 `T6` 已完成
- `V1 Done` 条件已满足
- 代码已超出原始 V1 任务清单，进入 `T7 assisted skeleton`
- 当前 `assisted` 已进入受控应用分支：低风险输出可自动 apply，`PlotTextHint` 仅允许人工批准 + 二次护栏 + 显式命令 + 快照校验后写入
- 当前验证基线：
  - `cargo test -q` 通过
  - 前端定向测试通过
  - `npm run build` 通过

## 2. 总任务图

当前建议拆成 8 个任务包：

1. `T0` 文档与入口稳定化 `已完成`
2. `T1` Core 类型与配置骨架 `已完成`
3. `T2` Trace 与图执行骨架 `已完成`
4. `T3` Capability 与协议对象骨架 `已完成`
5. `T4` 记忆与上下文骨架 `已完成`
6. `T5` DirectorAgent observe-only 接入 `已完成`
7. `T6` Guardrail 接入与前端调试最小闭环 `已完成`
8. `T7` Assisted Skeleton 与受控应用预备 `进行中`

## 3. 任务清单

## T0 文档与入口稳定化 `已完成`

### 目标

确保后续实现时，团队可以清楚知道该看哪些文档，避免边开发边重新找设计入口。

### 涉及文件

- `docs/README.md`
- `docs/architecture/README.md`
- `docs/architecture/noname-v1-blueprint.md`
- `.kiro/README.md`
- `.kiro/specs/Nobody/README.md`

### 子任务

- `T0-1` 确认 `docs/` 与 `.kiro/` 职责边界
- `T0-2` 确认 `NoName Agent` 四份主文档的阅读顺序
- `T0-3` 固定 V1 蓝图为实现基线

### 验收标准

- 新人可在 10 分钟内找到 `NoName Agent` 的全部正式设计文档
- 不再出现“正式文档和本地草稿混用”的情况

### 依赖

- 无

## T1 Core 类型与配置骨架 `已完成`

详见: `noname-t1-file-checklist.md`

### 目标

建立 NoName Agent 最基础的类型系统、配置体系和错误体系。

### 建议新增文件

- `src-tauri/src/noname_types.rs`
- `src-tauri/src/noname_config.rs`
- `src-tauri/src/noname_errors.rs`

### 子任务

#### `T1-1` 定义核心枚举与基础对象

至少包含：

- `NoNameMode`
- `NoNameProposalKind`
- `NoNameTraceStage`
- `NoNameRole`

#### `T1-2` 定义统一信封对象

至少包含：

- `NoNameEnvelope`
- `trace_id`
- `session_id`
- `kind`
- `payload`
- `meta`

#### `T1-3` 定义配置对象

至少包含：

- mode
- token budgets
- timeout policy
- capability whitelist
- trace retention size

#### `T1-4` 定义统一错误体系

至少包含：

- `NoNameProtocolError`
- `NoNameCapabilityError`
- `NoNameMemoryError`
- `NoNameContextError`
- `NoNameGuardrailError`

### 验收标准

- 所有核心对象都可序列化/反序列化
- 不依赖前端即可在 Rust 单测中构造完整对象
- 错误对象可统一映射为字符串或结构化错误载荷

### 依赖

- `T0`

## T2 Trace 与图执行骨架 `已完成`

### 目标

建立可追踪、可扩展的最小运行时主循环。

### 建议新增文件

- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_graph.rs`
- `src-tauri/src/noname_runtime.rs`

### 子任务

#### `T2-1` 定义 Trace 数据结构

至少包含：

- traceId
- turnId
- mode
- graphPath
- capabilityCalls
- guardrailResult
- fallbackUsed
- elapsedMs

#### `T2-2` 实现最小图执行器

最小节点建议：

- `CollectTurnInput`
- `BuildContextBundle`
- `PlanTurn`
- `PersistTrace`

#### `T2-3` 实现 Runtime 外壳

提供：

- `run_turn()`
- `store_trace()`
- `get_recent_traces()`
- `clear_traces()`

#### `T2-4` 提供最小调试命令接口

后续会接入 `lib.rs` / `tauri_commands.rs`。

### 验收标准

- 可以执行一次“空回合”并产出 trace
- 图执行器支持按节点记录路径
- Runtime 即使没有真实 Agent，也能完整跑完骨架流程

### 依赖

- `T1`

## T3 Capability 与协议对象骨架 `已完成`

### 目标

把工具、资源、提示模板统一纳管，并建立本地版 `NNCP-T / NNCP-A` 对象模型。

### 建议新增文件

- `src-tauri/src/noname_capability_base.rs`
- `src-tauri/src/noname_capability_registry.rs`
- `src-tauri/src/noname_resources.rs`
- `src-tauri/src/noname_prompt_catalog.rs`
- `src-tauri/src/noname_protocol_types.rs`
- `src-tauri/src/noname_protocol_tool.rs`
- `src-tauri/src/noname_protocol_agent.rs`

### 子任务

#### `T3-1` 定义 Capability Descriptor

至少支持三类：

- `ToolCapability`
- `ResourceCapability`
- `PromptCapability`

#### `T3-2` 建立本地 Capability Registry

提供：

- 注册
- 列表
- 按 ID 查找
- 调用/读取/解析入口

#### `T3-3` 定义 `NNCP-T` 对象

至少包含：

- Tool call
- Tool result
- Resource read
- Prompt resolve

#### `T3-4` 定义 `NNCP-A` 对象

至少包含：

- Agent address
- Agent message
- Task lifecycle

### 验收标准

- 能列出本地 capability
- 能用统一接口读取资源和解析 prompt
- 协议对象可以写入 trace

### 依赖

- `T1`
- `T2`

## T4 记忆与上下文骨架 `已完成`

### 目标

把现有 `memory_layers.rs` 和 `context_builder.rs` 升级成 NoName Agent 可用的记忆与上下文底座。

### 建议新增文件

- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_memory_store.rs`
- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_note_store.rs`

### 建议修改文件

- `src-tauri/src/memory_layers.rs`
- `src-tauri/src/context_builder.rs`

### 子任务

#### `T4-1` 定义四层记忆对象

- WorkingMemory
- EpisodicMemory
- SemanticMemory
- NarrativeMemory

#### `T4-2` 建立 MemoryManager

至少支持：

- write
- upsert
- query
- compact
- fetch_for_context

#### `T4-3` 补 NarrativeMemory

最小支持：

- chapter goal
- active conflict
- unresolved thread
- character arc note

#### `T4-4` 建立 GSSC 上下文流水线

最小支持：

- Gather
- Score
- Select
- Structure
- Compress

#### `T4-5` 产出 DirectorAgent 专属 Context Packet

### 验收标准

- 单回合可从多来源构建结构化上下文
- token budget 生效
- 不同角色可获得不同上下文包

### 依赖

- `T1`
- `T2`
- `T3`

## T5 DirectorAgent observe-only 接入 `已完成`

### 目标

在不改变现有剧情结果的前提下，让 `DirectorAgent` 实际参与一回合流程。

### 建议新增文件

- `src-tauri/src/noname_roles.rs`
- `src-tauri/src/noname_prompts.rs`
- `src-tauri/src/noname_tools.rs`

### 建议修改文件

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/lib.rs`

### 子任务

#### `T5-1` 定义 `DirectorAgent`

最小职责：

- 接受 Context Packet
- 输出 `NoNameProposal`
- 记录 rationale

#### `T5-2` 定义最小 prompt 模板

重点约束：

- 不直写最终状态
- 只输出结构化提案
- 遵守当前章目标与硬约束

#### `T5-3` 提供最小 capability

至少接入：

- `GeneratePlotCandidate`
- `ReadPlotSnapshot`
- `ReadWorldFacts`
- `ReadRecentEvents`

#### `T5-4` 在 `execute_player_action` 中接入 observe-only 分支

### 验收标准

- 一次玩家行动可触发 Agent proposal
- proposal 可记录到 trace
- 经典链路输出不变

### 依赖

- `T1`
- `T2`
- `T3`
- `T4`

## T6 Guardrail 接入与前端最小调试闭环 `已完成`

### 目标

让 Agent 结果可被校验，并且前端能查看最小调试信息。

### 建议新增文件

- `src-tauri/src/noname_guardrails.rs`
- 可新增 `src/components/AgentTracePanel.vue`

### 建议修改文件

- `src-tauri/src/tauri_commands.rs`
- `src/types/game.ts`
- `src/stores/gameStore.ts`
- `src/components/GameInfoCenterDialog.vue`

### 子任务

#### `T6-1` 建立 Guardrail Gateway

统一接入：

- `plot_consistency.rs`
- `state_patch_validator.rs`
- `entity_validator.rs`
- `numeric_guard.rs`

#### `T6-2` 建立 accept / reject / repair 结果对象

#### `T6-3` 暴露 trace / mode / diagnostics 读取接口

#### `T6-4` 前端显示最近 trace 和调试信息

### 验收标准

- Agent proposal 可以被校验
- reject reason 可见
- 前端开发模式可查看 trace 摘要

### 依赖

- `T1`
- `T2`
- `T3`
- `T4`
- `T5`

## T7 Assisted Skeleton 与受控应用预备 `进行中`

详见: `noname-t7-file-checklist.md`

### 目标

在不破坏经典主链路的前提下，让 `NoName Agent` 具备进入 `assisted` 模式的最小能力，为后续“受控应用 proposal”做准备。

### 已完成部分

- `NoNameMode::Assisted` 已接入配置预设
- runtime 可按模式区分 `observe_only` 与 `assisted`
- `DirectorAgent` 已输出结构化 `NoNameProposal`
- trace 已记录 proposal、guardrail 结果、fallback
- 后端已暴露 `get_noname_mode / set_noname_mode`
- 前端调试文本已显示 proposal 状态
- `NoNameProposal.status` 已落地
  - `observed / ready / blocked / applied / fallback`
- runtime 已按 guardrail 结果标记 proposal 为 `observed / ready / blocked`
- 诊断文本已输出 `proposal_status=...`
- trace 已记录 apply preflight 结果
- trace 已记录 controlled output review，能区分 `Allow / Reject / NeedsReview`
- 前端调试文本已显示 apply preflight 与 proposal transition log
- 前端调试台已展示 controlled output review，并支持复制当前 Trace 摘要
- A3 `forbiddenScopes` 已映射到 controlled output policy，review trace 会显示 `policyForbiddenScopes`
- apply 阶段已补独立 guardrail 入口
- `T7-4` 已补 `disabled / observe_only / assisted` 模式矩阵测试，覆盖 disabled 跳过、observe-only 只记录、assisted 受控低风险 apply
- `NeedsReview` 已补前端本地人工复核入口，可标记“可进入高层 apply 设计 / 暂不应用 / 重置待复核”，但不直接写入剧情正文
- `mark_noname_controlled_output_review` 已接入，人工复核 intent 会写回最近 trace 并记录状态迁移
- 人工批准后的 `PlotTextHint` intent 已进入二次 guardrail / apply planner 等待队列，trace 会记录 `review_intent_ready / awaiting_second_guardrail`
- `resolve_noname_second_guardrail` 已接入，能记录 `allow / reject / fallback` 二次护栏决策，但仍不直接写入剧情正文
- `T7-0.6` 已接入 `apply_noname_manual_plot_text_hint`，只有人工批准 + 二次护栏 allow + 当前章节/段落快照一致时，才允许显式写入 `PlotTextHint`
- `T7-0.7` 已补显式人工 apply 的前端差异预览、重复写入禁用与 stale snapshot 友好提示
- `T7-3` 已补 apply lifecycle 展示与测试基线，调试面板、复制摘要、Info 调试文本可以统一展示 apply / review / guardrail / manual apply / fallback 进度
- `T7-1` 第七切片已新增 `noname_apply.rs` reviewed apply runtime，提供 `apply_noname_reviewed_output` 通用命令入口；现有 `PlotTextHint` 显式写入已改走该入口，`ChapterSummaryHint`、`OptionBiasHint` 与 `PlotAugmentationHint` 也已接入人工批准 + 二次护栏 + 快照一致的显式 reviewed apply，并把低风险/非最终提示状态写入下沉到 `plot_engine`；pending augmentation 已能作为安全上下文进入下一轮 plot prompt，前端调试台已可显式触发四类 scope 的 reviewed apply

### 当前边界

- proposal 目前不会自动改写剧情正文与状态机，但已可写入章节摘要提示、选项偏置提示与 pending 剧情增强提示这类非最终输出
- 当前已完成 `assisted preflight`，并可应用到“诊断层 + 章节摘要提示 + 选项偏置提示 + pending 剧情增强提示”四类受控输出；`plotTextHint` 仍会先进入 controlled output review、人工复核 intent、二次护栏，再由显式人工命令写入当前剧情段落
- reviewed apply runtime 已形成统一通道，目前支持 `PlotTextHint`、`ChapterSummaryHint`、`OptionBiasHint` 与 `PlotAugmentationHint`；`plot_engine` 已提供章节摘要提示、诊断提示与 pending augmentation 的受控输出层入口，并能在下一轮 prompt 中安全消费 pending augmentation
- 仍然保持“经典主链路优先，NoName 仅辅助”

### 下一步子任务

- `T7-1.2` 已完成两个扩展 scope：`ChapterSummaryHint` 与 `OptionBiasHint` 可复用 reviewed apply runtime，并已通过 `plot_engine` 低风险输出层落地；前端调试台已补显式 apply 入口和快照预览
- `T7-1.3` 已完成 non-final plot augmentation 最小接入：`PlotAugmentationHint` 可复用 reviewed apply runtime，要求 pending 列表快照一致后写入 `pending_plot_augmentation_hints`，并已接入前端调试台预览与 Web mock
- `T7-1.4` 已完成 pending augmentation 安全消费：下一轮 plot prompt 会注入非最终安全上下文，成功由 plot_engine 生成并落地后按快照清空；quick mode、预设回退或双通道叙事覆盖时会保留 pending hints

### 验收标准

- `assisted` 模式下 proposal 可以进入受控应用预备分支
- 未通过 guardrail 的 proposal 不得影响主剧情结果
- 任何情况下都可 fallback 到经典链路
- 前端调试信息能明确显示 proposal 当前状态

## 4. 建议迭代节奏

建议按提交切片推进，而不是一次性堆大改动。

### Iteration A

- 完成 `T1`
- 完成 `T2`

### Iteration B

- 完成 `T3`

### Iteration C

- 完成 `T4`

### Iteration D

- 完成 `T5`

### Iteration E

- 完成 `T6`

### Iteration F

- 推进 `T7`

## 5. 建议提交切片

建议至少拆为以下提交：

1. `feat(noname): 新增核心类型与配置骨架`
2. `feat(noname): 新增 trace 与最小图执行器`
3. `feat(noname): 新增 capability registry 与协议对象`
4. `feat(noname): 新增记忆与上下文骨架`
5. `feat(noname): 接入 DirectorAgent observe-only 模式`
6. `feat(noname): 接入 guardrail gateway 与前端调试入口`
7. `feat(noname): 新增 assisted skeleton 与 proposal applyable 标记`

## 6. 测试清单

### 单元测试

- `noname_types` 的序列化/反序列化
- `noname_graph` 的节点跳转
- `noname_capability_registry` 的注册与发现
- `noname_context_builder` 的 gather/score/select/structure/compress
- `noname_guardrails` 的 accept/reject/repair

### 集成测试

- `execute_player_action` 在 `disabled` 下不变
- `observe_only` 能产出 trace
- Agent 失败自动 fallback
- `assisted` 仅在 guardrail 通过时标记 proposal 为 `ready`

### 前端测试

- store 能读取 trace
- 调试面板仅在开发模式下展示
- 调试信息不会破坏现有 GameInfoCenterDialog 行为

## 7. Done 定义

当以下条件全部成立时，可以认为 `NoName Agent V1` 基础闭环已经完成：

- 后端具备 `NoName Runtime` 骨架
- 有统一 Envelope / Task / Capability / Trace 对象
- 有最小记忆与上下文流水线
- `DirectorAgent` 已接入 `execute_player_action`
- 支持 `observe_only`
- Guardrail 已接入
- 前端可以查看最近 trace
- 经典主链路仍可正常运行

当前状态：上述条件已经满足，`NoName Agent V1` 可视为已完成。

## 8. 推荐下一步

如果继续推进实现，当前最推荐的是：

1. 继续评估是否需要更高一层的受控输出类型，例如 non-final plot augmentation，但仍不得直接接管最终剧情正文。
2. 把后续新增 scope 持续接入现有 apply lifecycle，让更多输出类型复用同一套可视化与测试基线。
这会比继续扩文档或继续堆更多角色更有价值，因为它决定 `NoName Agent` 是否能从“可观察”走向“可辅助落地”。

## 2026-04-18 T7-1.5 Progress Note

- `T7-1.5` has completed the pending plot augmentation observability slice.
- `PlotAugmentationHint` remains a non-final, reviewed, manually staged hint. The next generation may consume it, but final plot state mutation is still owned by `plot_engine`.
- The consume/retain result is now visible in trace execution logs, proposal transition logs, frontend lifecycle summaries, Web mock, and focused tests.
- Recommended next direction after T7-1.5: review whether more controlled output scopes need the same pending-consume lifecycle pattern, or move to a broader trace UX cleanup instead of adding new plot authority.

## 2026-04-18 T7-1.6 Progress Note

- `T7-1.6` has completed the first trace UX cleanup slice for pending plot augmentation.
- Debug copy reports and `gameStore` Info debug text now show a compact plot augmentation summary, making `已消费 / 已保留 / 待消费 / 待观察 / 无` readable without opening raw JSON.
- No plot authority was expanded in this slice; it only improves trace interpretation and operator visibility.

## 2026-04-19 T7-1.7 Progress Note

- `T7-1.7` extends the same trace UX cleanup to the visible `AgentTracePanel` overview.
- The pending plot augmentation summary now appears in the panel, debug copy report, and Info debug text through one shared helper.
- This keeps the assisted apply observability path consistent while preserving the non-final prompt-context boundary.

## 2026-04-19 T7-1.8 Progress Note

- `T7-1.8` improves execution-log readability for pending plot augmentation records.
- `AgentTracePanel` now shows readable execution labels such as `剧情增强提示 · 已消费`, while keeping the original trace target/outcome visible for low-level debugging.
- No runtime behavior changed; this is an operator-facing trace interpretation slice only.

## 2026-04-19 T7-1.9 Progress Note

- `T7-1.9` extends the readable apply execution mapping to `gameStore` Info text and `InfoTabsDialog`.
- Operator-facing debug surfaces now consistently translate pending plot augmentation execution records while preserving raw target/outcome details.
- This remains a trace UX cleanup slice and does not alter assisted apply runtime behavior.

## 2026-04-19 T7-2.0 Progress Note

- `T7-2.0` extends readable apply execution summaries to copied `NoNameDebugConsole` trace reports.
- Copy/paste debugging now includes both readable execution labels and raw trace target/outcome for translated pending plot augmentation records.
- Runtime behavior and safe-output authority remain unchanged.

## 2026-04-19 T7-2.1 Progress Note

- `T7-2.1` centralizes apply execution summary formatting in `summarizeNoNameApplyExecutions`.
- Info debug text and copied debug-console reports now share one execution-summary implementation with configurable raw/note formatting.
- This reduces future drift across debug surfaces when new execution outcomes are added.

## 2026-04-19 T7-2.2 Progress Note

- `T7-2.2` completed a focused full-regression verification pass for the current T7 line.
- Backend T7 tests, clippy, frontend NoName trace tests, frontend build, and diff whitespace checks passed.
- The only build note remains the existing Vite dynamic-import warning for `@tauri-apps/api/core.js`; no new blocker was found.

## 2026-04-19 T7-2.3 Progress Note

- `T7-2.3` cleaned the T7 status language so optional future expansion is no longer described as unfinished core work.
- T7 can now be treated as functionally complete pending final pre-PR review/commit preparation.
