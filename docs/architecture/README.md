# Architecture Docs Index

本目录保存 `Nobody` 的架构、领域模型与 `NoName Agent` 相关正式设计文档。

## 当前状态

截至 `2026-04-18`：

- `NoName Agent V1` 基础闭环已完成
- 当前代码进度已经超过原始 `T6` 计划，进入 `T7 assisted skeleton / 受控应用 proposal` 阶段
- `T7-0.6` 已完成：`PlotTextHint` 只有在人工批准、二次护栏 allow、显式命令与段落快照一致时才会写入正文
- `T7-0.7` 已完成：前端调试台已补显式人工 apply 的差异预览、重复写入禁用与 stale snapshot 友好提示
- `T7-3` 已完成最小可视化基线：apply lifecycle 已统一展示到调试面板、复制摘要与 Info 调试文本
- `T7-1` 已完成第七切片：后端 reviewed apply runtime 与通用命令入口 `apply_noname_reviewed_output` 已复用到 `PlotTextHint`、`ChapterSummaryHint`、`OptionBiasHint` 与 `PlotAugmentationHint`；`PlotAugmentationHint` 会进入 `pending_plot_augmentation_hints`，并已能作为“非最终、可忽略”的安全上下文进入下一轮 plot prompt，成功由 plot_engine 消费后按快照清空
- 如果要继续推进实现，优先阅读 `noname-v1-blueprint.md`、`noname-v1-task-list.md` 与 `noname-t7-file-checklist.md`

## 推荐阅读顺序

### 基础架构

1. `../ARCHITECTURE.md`
2. `domain-model-v2.md`

### NoName Agent 主线

1. `noname-agent-v1.md`
2. `noname-memory-context-v1.md`
3. `noname-framework-protocol-v1.md`
4. `noname-v1-blueprint.md`
5. `noname-v1-task-list.md`
6. `noname-t1-file-checklist.md`
7. `noname-t7-file-checklist.md`
8. `noname-collaboration-handoff-20260413.md`
9. `noname-collaboration-cards/README.md`

### 历史补充

- `agent-framework-v1.md`

## 文件定位

- `domain-model-v2.md`
  - 领域实体、不变量、生命周期与当前模块映射。

- `noname-agent-v1.md`
  - NoName Agent 的总体框架、角色体系、状态图与模块边界。

- `noname-memory-context-v1.md`
  - 记忆工程、上下文工程、GSSC 流水线、长时程策略。

- `noname-framework-protocol-v1.md`
  - 框架分层、Capability 抽象、NNCP 协议体系。

- `noname-v1-blueprint.md`
  - 面向实现的 V1 蓝图、阶段目标、当前状态与下一阶段方向。

- `noname-v1-task-list.md`
  - 可执行的开发任务清单，现已补充 `T0-T6` 完成状态与 `T7 assisted skeleton` / 显式人工 apply 说明。

- `noname-t1-file-checklist.md`
  - `T1 Core 类型与配置骨架` 的文件级实现清单，明确每个文件该放什么、先做什么、做到什么算完成。

- `noname-t7-file-checklist.md`
  - `T7 Assisted Skeleton` 的文件级实现清单，聚焦 proposal 进入受控应用分支前的模块拆解与落点。

- `noname-collaboration-handoff-20260413.md`
  - 面向协作者的任务拆分文档，明确本机已完成内容、主线进行中内容，以及“尚未开始、适合外包协作”的任务边界。

- `noname-collaboration-cards/README.md`
  - 面向协作者的任务卡片索引，一项任务一个文件，可直接转发。

- `agent-framework-v1.md`
  - 早期 Agent 接入提案，保留供追溯参考。

## 维护规则

- 与 `NoName Agent` 相关的新设计优先补充到对应专题文档。
- 如果内容跨越多个专题，先更新 `noname-v1-blueprint.md` 的执行计划，再回写细节文档。
- 如果内容进入实施层，优先更新 `noname-v1-task-list.md`。
- 如果内容已经下钻到文件级或提交级，补充到对应专项清单，如 `noname-t1-file-checklist.md` 与 `noname-t7-file-checklist.md`。
- 如果代码进度超出原始计划，先更新本索引中的“当前状态”，避免阅读者被旧计划误导。
- 废弃文档不直接删除，先在本索引中标记为“历史补充”。

## 2026-04-18 T7-1.5 Update

- Pending plot augmentation observability has been closed: runtime traces now record `pending_plot_augmentation_consumed` or `pending_plot_augmentation_retained` in `applyExecutionLog`, proposal transition logs include the same lifecycle signal, and the frontend apply lifecycle surfaces this as `剧情增强消费`.
- Web mock behavior and focused tests were updated so non-quick assisted generation consumes staged `PlotAugmentationHint` entries, while quick-mode retains them with an explicit reason.

## 2026-04-18 T7-1.6 Update

- Trace UX cleanup has started with a compact pending plot augmentation summary: debug copy reports now include `Plot Augmentation: ...`, and Info debug text includes `剧情增强提示：...`.
- This keeps the operator-facing view aligned with lifecycle details while preserving the existing safe boundary: pending augmentation remains non-final prompt context only.

## 2026-04-19 T7-1.7 Update

- The compact pending plot augmentation summary is now visible in `AgentTracePanel` run overview as well, so the trace panel, debug copy report, and Info debug text all describe the same consumed/retained/staged state.

## 2026-04-19 T7-1.8 Update

- `AgentTracePanel` now translates pending plot augmentation execution rows into readable labels while preserving the original target/outcome line for debugging.

## 2026-04-19 T7-1.9 Update

- The same readable apply execution mapping now powers `gameStore` Info debug text and `InfoTabsDialog`, keeping all operator-facing debug surfaces aligned.

## 2026-04-19 T7-2.0 Update

- Copied `NoNameDebugConsole` trace reports now include readable apply execution summaries with raw target/outcome details for translated pending plot augmentation records.

## 2026-04-19 T7-2.1 Update

- Apply execution summary formatting is now centralized in `summarizeNoNameApplyExecutions`, shared by Info debug text and copied debug-console reports.

## 2026-04-19 T7-2.2 Update

- A focused full-regression pass for T7 apply/trace UX completed successfully across Rust tests, clippy, frontend NoName trace tests, frontend build, and diff whitespace checks.

## 2026-04-19 T7-2.3 Update

- T7 documentation wording was cleaned up: current remaining items are now framed as optional future extensions rather than unfinished core functionality.

## 2026-04-19 T8-1 Update

- The next NoName line has started with multi-role observe fan-out cleanup: `NoNameRuntime` now specializes context per role before dispatching non-Director agents through the protocol runtime.
- This keeps assisted apply authority unchanged while making `WorldCurator / NpcIntent / CombatNarrator` observe results depend on role-specific context slices.

## 2026-04-19 T8-2 Update

- Multi-role observe prompts and tool args now receive `roleGoal / forbiddenScopes`, so role-specific boundaries are visible inside the local prompt/tool pipeline as well as the protocol delegation events.

## 2026-04-19 T8-3 Update

- `NoNameAgentRegistry` now accepts `NoNameRoleContextPacket` directly for observe dispatch, so runtime fan-out no longer owns the flattened-context compatibility adapter.

## 2026-04-19 T8-4 Update

- Multi-role related observations now expose role context summaries in trace/debug surfaces, including role goal, scene focus, and forbidden scopes.

## 2026-04-19 T8-5 Update

- Structured notes now influence role context ordering: active note types are ranked per role before entering narrative context and chapter-summary context.
- This connects A2 structured notes to the multi-role observe fan-out while keeping NoName in the observe-only boundary.

## 2026-04-19 T8-6 Update

- Related observations and role-context debug summaries now expose `noteTypeHits`, so operators can see which structured note types were prioritized for each role.
- This makes the A2 -> T8 role-context ranking path observable in trace panel, copied debug reports, and store-level debug text.

## 2026-04-19 T8-7 Update

- Related observations now also expose role-context `sourceStats` and `contextTokenBudgetUsed`, so operators can compare both note-priority hits and upstream context shape.
- Trace panel, copied debug-console reports, and store-level debug text now show those compact source summaries without changing runtime authority boundaries.

## 2026-04-20 T8-8 Update

- Role-context packets now emit `contextSliceStats`, summarizing how each role-specific section was trimmed from source-count to visible-count.
- Trace/debug surfaces now show compact slice deltas such as `worldFacts:4->3`, making per-role context clipping easier to audit.
