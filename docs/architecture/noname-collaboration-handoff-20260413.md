# NoName Agent 协作拆分文档

更新时间: 2026-04-13
状态: 协作者任务抽离版
目标: 梳理 `NoName Agent` 当前完成情况，明确本机已完成内容，并抽离“尚未开始、适合交给协作者”的任务清单
关联文档:
- `noname-v1-blueprint.md`
- `noname-v1-task-list.md`
- `noname-t7-file-checklist.md`
- `noname-memory-context-v1.md`

## 1. 这份文档的用途

这份文档不再讨论总体理念，而是回答三个很实际的问题：

- 当前这台机器上，`NoName Agent` 已经完成到了哪一步
- 哪些任务已经在主线中推进，暂时不适合交给协作者并行开发
- 哪些任务目前还没有开始，且可以相对独立地交给协作者

适用场景：

- 你准备把一部分 `NoName Agent` 工作交给协作者
- 你需要避免协作者改到当前主线正在推进的文件
- 你希望协作者从“未启动工作”开始，而不是接手半成品主线

## 2. 当前状态总览

结合当前代码和既有文档，`NoName Agent` 的状态应理解为：

- `V1` 基础骨架已完成
- `T0-T6` 可视为完成
- `T7` 已经明显超出文档中 `2026-04-09` 的描述
- 当前代码已经进入“低风险受控 apply + planner 可视化”阶段
- 当前仍未进入“真正影响主剧情结果”的高权重受控应用阶段

换句话说：

- 现在的 `NoName` 已经不是“只会 observe-only”
- 但也还不是“完整可控地接管剧情生成的一部分”

## 3. 本机已完成任务

以下内容已经在本机代码中落地，协作者不应再从这些任务重新起步。

### 3.1 V1 骨架层

- `NoName` 核心类型、配置、错误、trace、runtime、graph 骨架已完成
- 本地 capability registry、resource、prompt、protocol 对象已完成
- 记忆与上下文骨架已完成
- `DirectorAgent` 已接入主链
- `observeOnly` 模式已完成
- guardrail gateway 已接入
- 前端调试面板已可查看 trace / proposal / guardrail / fallback

### 3.2 T7 已完成部分

- `NoNameMode`
  - `disabled / observeOnly / assisted`
- `NoNameProposal`
  - `status`
  - `apply_scopes`
  - `target_segment`
  - `intended_effect`
- runtime apply preflight
  - `ready / blocked / fallback`
- apply trace
  - `proposal_transition_log`
  - `apply_plan_log`
  - `apply_execution_log`
- 低风险 apply 已接入
  - `plot_text_hint`
  - `chapter_summary_hint`
  - `option_bias_hint`
- planner 已具备
  - `decision`
  - `priority`
  - `order`
- web mock 已同步支持
- 前端 debug 面板已显示
  - proposal 状态
  - 目标段
  - 预期效果
  - 作用域
  - 应用计划
  - 应用执行

### 3.3 当前主线涉及文件

这些文件已经深度参与当前 `T7` 主线，不建议协作者直接接手：

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src-tauri/src/noname_types.rs`
- `src/platform/webRuntime.ts`
- `src/stores/gameStore.ts`
- `src/components/InfoTabsDialog.vue`
- `src/components/GameInfoCenterDialog.vue`
- `src/types/game.ts`

原因：

- 这些文件已经进入“持续演化中的主链编排层”
- 当前逻辑互相耦合较强
- 协作者如果从这些文件切入，容易和本机主线发生交叉修改

## 4. 本机正在推进但暂不建议拆出的工作

以下任务虽然还没有最终收口，但已经开始推进，不适合当作“未开始任务”交给协作者。

### 4.1 T7 主线收口

- 将 apply planner 继续演化为真正的冲突裁决器
- 将 apply 逻辑从 `tauri_commands.rs` 抽离为独立模块，例如 `noname_apply.rs`
- 增加 apply 后二次 guardrail 与 fallback
- 收敛统一的 apply 结果语义与 diagnostics 文案

这些任务与当前主链高度相关，应优先由本机继续推进。

## 5. 可交给协作者的任务边界

协作者应优先选择：

- 尚未开始
- 与当前主线低耦合
- 能独立定义输入输出
- 改动文件范围相对清晰

不建议协作者直接接手：

- 当前 `T7` 的 apply 主线编排
- `execute_player_action` 主入口
- 当前 trace 主结构的持续演化

## 6. 协作者任务清单

下面只列“尚未开始”或“可视为尚未开始”的任务。

## A 清单

特点：

- 任务量中等
- 与 `NoName` 主线联系较高
- 但改动范围可以相对隔离
- 完成后能直接服务后续主线

### A1 记忆压缩与长期整理模块

状态: 未开始

目标：

- 新增 `NoName` 的 compaction 能力
- 对长历史、章节历史、trace 历史做结构化压缩

建议文件：

- `src-tauri/src/noname_memory_compaction.rs`
- `src-tauri/src/noname_memory_manager.rs`
- `src-tauri/src/noname_note_store.rs`

建议交付：

- turn compaction
- chapter compaction
- trace compaction
- 最小单元测试

为什么适合协作者：

- 这部分在设计文档里定义充分
- 当前主线还未启动
- 改动重心在记忆层，不直接碰 apply 主线

### A2 Structured Notes / Narrative Notes 增强

状态: 未开始

目标：

- 把 `NarrativeMemory` 从基础骨架推进到真正可用的结构化笔记层

建议文件：

- `src-tauri/src/noname_note_store.rs`
- `src-tauri/src/noname_memory_types.rs`
- `src-tauri/src/noname_memory_store.rs`

建议交付：

- note type 扩展
  - `goal`
  - `conflict`
  - `foreshadowing`
  - `unresolved_thread`
  - `character_arc`
- note 的 `active / resolved / archived`
- 章节结束时的 note 整理接口

### A3 角色差异化上下文包

状态: 未开始

目标：

- 在现有 `Context Packet` 基础上，支持不同角色拿到不同上下文

建议文件：

- `src-tauri/src/noname_context_builder.rs`
- `src-tauri/src/noname_context_types.rs`
- `src-tauri/src/noname_roles.rs`

建议交付：

- `Director` 专属上下文再细化
- `WorldCurator` 上下文初版
- `NpcIntent` 上下文初版
- 相应的 context builder 单测

说明：

- 这项和主线联系高，但目前还没启动多角色落地
- 可以由协作者先把上下文分型做好，为后续角色扩展打基础

### A4 受控高一层输出接口预研

状态: 未开始

目标：

- 为后续“比低风险输出更进一步”的受控应用准备接口
- 但不直接接管最终剧情状态机

建议文件：

- `src-tauri/src/plot_engine.rs`
- 可新增设计草案文档

建议交付：

- 列出允许受控 apply 的下一层输出
- 定义不可碰范围
- 提供接口草案与最小测试草案

说明：

- 这项应由协作者做“接口设计与隔离层”
- 不建议协作者直接修改当前 `tauri_commands.rs` 主线 apply

## B 清单

特点：

- 任务量较大
- 与当前主线联系较低或中等
- 更适合作为独立支线推进

### B1 多角色 Agent 扩展

状态: 未开始

目标：

- 从单一 `DirectorAgent` 扩展到多角色协作

建议方向：

- `WorldCuratorAgent`
- `NpcIntentAgent`
- `CombatNarratorAgent`

说明：

- 当前设计里有位置，但代码主线还没正式开始
- 这部分可以独立开发，不会卡住当前 `T7`

### B2 记忆检索增强

状态: 未开始

目标：

- 在 SQLite 基础上增强检索排序、标签检索、角色专属召回

建议文件：

- `src-tauri/src/noname_memory_retrieval.rs`
- `src-tauri/src/noname_memory_store.rs`

可交付项：

- by_actor / by_location / by_goal / by_keyword 检索
- relevance / recency / importance 排序增强

### B3 独立 Agent 调试台

状态: 未开始

目标：

- 新增更独立的 `NoName Agent` 调试控制台，而不是仅依赖当前信息抽屉

建议文件：

- `src/components/AgentTracePanel.vue`
- `src/components/NoNameDebugConsole.vue`
- `src/stores/gameStore.ts`

说明：

- 当前前端已有最小调试入口
- 更强 UI 仍未开始，适合作为协作者支线

### B4 真实协议/通信层增强

状态: 未开始

目标：

- 把当前本地版 `NNCP-T / NNCP-A` 从对象模型推进到更真实的通信层

建议文件：

- `src-tauri/src/noname_protocol_agent.rs`
- `src-tauri/src/noname_protocol_tool.rs`
- `src-tauri/src/noname_protocol_types.rs`

说明：

- 当前已有本地协议骨架
- 真正的 agent-to-agent / task lifecycle 增强还未开始

### B5 外部知识与高级检索后端

状态: 未开始

目标：

- 为未来 lore / wiki / 外设定集接入做基础

建议方向：

- 向量检索
- 图检索
- 外部 lore RAG

说明：

- 这部分不适合当前主线优先推进
- 但很适合作为协作者的独立中长期分支

## 7. 协作者不应直接接手的文件

为了避免和本机主线冲突，协作者应避免直接修改以下文件：

- `src-tauri/src/tauri_commands.rs`
- `src-tauri/src/noname_runtime.rs`
- `src-tauri/src/noname_guardrails.rs`
- `src-tauri/src/noname_trace.rs`
- `src/platform/webRuntime.ts`
- `src/components/InfoTabsDialog.vue`
- `src/stores/gameStore.ts`

如果协作者任务必须触碰这些文件，建议只通过小接口接入，并在合并前单独 review。

## 8. 推荐分配方式

推荐把任务这样分配：

- 本机继续负责
  - `T7` apply 主线
  - planner / executor / guardrail / fallback 收口
  - 经典链路与 assisted 主链一致性

- 协作者优先负责
  - `A1` 记忆压缩
  - `A2` structured notes
  - `A3` 角色差异化上下文
  - `B3` 独立调试台

这样分配的好处：

- 主线不被打断
- 协作者有足够独立的工作面
- 后续成果可以自然回接到 `NoName` 主线

## 9. 推荐交接格式

如果要把任务发给协作者，建议每项任务都附：

- 目标
- 不要碰的文件
- 建议新增文件
- 交付标准
- 验证命令

推荐统一验证命令：

- `cargo test noname_ -- --nocapture`
- `npm run test -- --run src/platform/webRuntime.test.ts src/components/__tests__/InfoTabsDialog.test.ts src/components/__tests__/GameInfoCenterDialog.test.ts src/stores/__tests__/gameStore.test.ts`

## 10. 结论

当前 `NoName Agent` 的最佳协作策略不是把正在推进的 `T7 apply` 主线拆给别人，而是：

- 本机继续收口 `T7`
- 把尚未开始、但与主线高度互补的记忆/上下文/多角色/独立调试任务交给协作者

这会比“多人同时改主链入口文件”稳定得多，也更容易合并成果。
