# NoName Role Context Packets V1

更新时间: 2026-04-16
对应任务: `A3-role-context-packets`

## 目标

在现有 `NoNameContextPacket` 基础上新增角色差异化上下文视图，让不同 `NoName Agent` 拿到符合自身职责的上下文切片，而不是共享同一个大包。

这版只做后端 builder / dispatcher，不接入 `noname_runtime` 主链，也不让前端直接依赖新结构。

## 新增结构

当前新增 `NoNameRoleContextPacket`:

- `role`
- `roleGoal`
- `sceneFocus`
- `worldFacts`
- `characterRelationships`
- `narrativePriorities`
- `recentSignals`
- `visibleConstraints`
- `forbiddenScopes`
- `sourceStats`
- `tokenBudgetUsed`

该结构不是替代原始 `NoNameContextPacket`，而是从基础包裁剪出的角色专属视图。

## Builder 入口

当前新增:

- `build_role_context_packet`
- `build_role_context_packets`
- `specialize_context_packet`

`build_role_context_packet` 会先复用现有 `build_context_packet`，再按角色做差异化裁剪。

## 当前角色视图

### Director

主要可见:

- narrative notes
- episodic memory
- chapter summaries
- recent signals

边界:

- 可以选择低风险剧情关注点
- 不能直接改写最终剧情状态
- 不能绕过 WorldCurator 发明硬设定

### WorldCurator

主要可见:

- hard facts
- chapter summaries
- referenced entities
- 少量 recent context

边界:

- 可以维护世界事实、地点约束和设定锚点
- 不能决定 NPC 私密意图
- 不能单独选择主剧情推进方向

### NpcIntent

主要可见:

- referenced entities
- episodic memory
- narrative notes
- recent context

边界:

- 可以推断 NPC 动机和关系压力
- 只能基于可见信息推断
- 不能泄露上下文中不存在的隐藏信息

### CombatNarrator

当前也提供初版视图，供后续复用:

- recent context
- episodic memory
- working memory
- hard facts

边界:

- 可以提供动作反馈和战斗叙事锚点
- 不能决定最终伤害、胜负或新增战斗规则

## 当前限制

- 角色上下文包已接入 runtime fan-out：runtime 现在生成 `NoNameRoleContextPacket` 并直接交给 agent registry。
- agent registry 当前内部仍通过 `flatten_role_context_packet` 兼容现有角色 agent，避免一次性重写全部 prompt/tool/resource pipeline。
- prompt/tool pipeline 已开始读取 `roleGoal / forbiddenScopes`；但当前仍通过 flattened context metadata 传递，不是最终的原生角色上下文接口。
- 当前差异化是规则式裁剪，不是 LLM 动态裁剪。
- `forbiddenScopes` 已开始接入受控输出策略，用于生成 `policyForbiddenScopes` trace；`visibleConstraints` 仍先作为显式文本约束。

## 后续建议

下一步可以继续:

1. 逐步让具体角色 agent / registry builder 原生消费 `NoNameRoleContextPacket`，移除内部 flattened context 适配层。
2. 继续细化 role context 摘要的 debug 体验，例如展示更细的裁剪原因或 token 预算分配依据。
3. 继续细化 note type 命中统计的呈现粒度，例如展示更多来源统计或排序前后的裁剪差异。

## 2026-04-19 T8-1 Update

- 新增 `flatten_role_context_packet`，用于把角色专属视图安全压回现有 `NoNameContextPacket`，让当前 agent registry 无需大改即可消费 role context。
- `NoNameRuntime` observe fan-out 已改为“基础 context -> role context specialization -> flattened agent context”的分发路径。
- 当前仍不接入 assisted apply；多角色结果继续作为 observe-only 相关观察写入 trace。

## 2026-04-19 T8-2 Update

- role observe prompt templates now include `roleGoal` and `forbiddenScopes` variables.
- `run_observe_capability_pipeline` fills those variables from flattened role context metadata and passes them into tool args as well.
- This keeps the current adapter shape but makes each role's prompt/tool execution aware of its role-specific boundary.

## 2026-04-19 T8-3 Update

- `NoNameAgentRegistry` now exposes `dispatch_role_context_observe_turn`, accepting `NoNameRoleContextPacket` directly.
- `NoNameRuntime` no longer calls `flatten_role_context_packet`; runtime fan-out only specializes context and delegates the compatibility adapter to the registry boundary.
- The internal flatten adapter remains temporary until individual role agents and registry builders can consume role context natively.

## 2026-04-19 T8-4 Update

- fan-out related observations now carry `roleGoal / sceneFocus / forbiddenScopes` into `NoNameTrace`.
- `AgentTracePanel`, copied debug-console reports, and `gameStore` debug text surface role context summaries so operators can compare what each role saw.
- This remains observe-only debug visibility; it does not grant multi-role agents assisted apply authority.

## 2026-04-19 T8-5 Update

- Active structured notes are now ranked per role before entering `narrative_notes` and `chapter_summaries`.
- Director prioritizes conflict/goal/thread notes, WorldCurator prioritizes goal/foreshadowing/thread notes, NpcIntent prioritizes character-arc/conflict/thread notes, and CombatNarrator prioritizes conflict/character-arc notes.
- This is the first direct A2 -> A3/T8 integration point inside context construction.

## 2026-04-19 T8-6 Update

- `NoNameRoleContextPacket` now carries `note_type_hits`, summarizing the top structured note types selected for each role after ranking.
- Observe fan-out copies those hit summaries into related observations and frontend debug surfaces, so operators can inspect why different roles received different note ordering.

## 2026-04-19 T8-7 Update

- Observe fan-out now also copies `source_stats` and `token_budget_used` into related observations.
- Frontend trace/debug surfaces render compact source summaries such as `semantic:3 / narrative:2` plus token budget, making role-context slicing easier to audit.

## 2026-04-20 T8-8 Update

- `NoNameRoleContextPacket` now carries `context_slice_stats`, recording section-level `source_count -> visible_count` deltas for role-specific context slices.
- Observe fan-out, trace records, and frontend debug summaries surface those deltas so operators can compare how much context each role retained per section.
