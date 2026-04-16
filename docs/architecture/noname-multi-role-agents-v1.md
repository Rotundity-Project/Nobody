# NoName Multi-Role Agents V1

更新时间: 2026-04-14
对应任务: `B1-multi-role-agents`

## 目标

在不改动当前 `NoNameRuntime` 主链的前提下，为 `NoName Agent` 增加一层可注册、可分发、可独立测试的多角色骨架。

这一版只解决三件事:

- 明确角色职责边界
- 提供最小 observe-only 执行 stub
- 提供统一 registry / dispatch 入口

## 当前角色划分

### Director

- 职责: 统筹当前回合最值得推进的剧情关注点
- 主要输入: `narrative_notes`, `episodic_memory`, `chapter_summaries`
- 输出: `PlotCandidate`
- 边界: 不直接补世界事实、NPC 动机或战斗节奏细节

### WorldCurator

- 职责: 补全世界事实、场景约束和设定锚点
- 主要输入: `hard_facts`, `chapter_summaries`, `referenced_entities`
- 输出: `WorldPatchProposal`
- 边界: 不负责决定主冲突推进方向

### NpcIntent

- 职责: 推断 NPC 动机、立场变化和关系反应
- 主要输入: `referenced_entities`, `recent_context`, `episodic_memory`
- 输出: `NpcIntentProposal`
- 边界: 不负责编排整段剧情，只补人物反应层

### CombatNarrator

- 职责: 观察冲突节奏、动作反馈与战斗描写锚点
- 主要输入: `recent_context`, `episodic_memory`, `action_summary`
- 输出: `CombatNarration`
- 边界: 不负责世界规则和 NPC 长线动机

## 设计取舍

- V1 初期先不接入 `noname_runtime.rs`
  - 当前已在 B4 中完成 observe fan-out 的最小接入，但 assisted apply 主链仍保持隔离
- 先保留每个角色自己的 prompt / tool / resource bundle
  - 便于后续独立演进，而不是把所有角色逻辑塞进 `DirectorAgent`
- 统一通过 `NoNameAgentRegistry` 做最小分发
  - 让后续 `B4 protocol` 或 `B3 debug console` 可以直接复用

## 后续接入建议

下一步如果要继续往前推进，建议顺序是:

1. 在 runtime 中增加“可选的多角色 observe fan-out”
2. 为每个角色补更真实的输入裁剪与 guardrail
3. 再决定哪些角色真正参与 assisted apply
