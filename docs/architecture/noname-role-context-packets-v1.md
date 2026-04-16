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

- 角色上下文包还没有接入 runtime fan-out，当前仍是独立 builder 能力。
- 当前差异化是规则式裁剪，不是 LLM 动态裁剪。
- `forbiddenScopes` 已开始接入受控输出策略，用于生成 `policyForbiddenScopes` trace；`visibleConstraints` 仍先作为显式文本约束。

## 后续建议

下一步可以继续:

1. 在 B4 observe fan-out 中改用 `build_role_context_packet` 或由基础包生成角色视图。
2. 让每个角色 prompt 继续读取 `roleGoal / forbiddenScopes`，并与受控输出策略保持一致。
3. 与 A2 structured notes 联动，让不同角色优先读取不同 note type。
