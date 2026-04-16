# NoName Memory Compaction V1

更新时间: 2026-04-16
对应任务: `A1-memory-compaction`

## 目标

为 `NoName Agent` 增加独立的记忆压缩层，用于把长回合文本、章节事件和 trace 噪声整理成可复用的结构化摘要。

这版不接入 `T7 apply` 主链，只提供记忆层能力和 `NoNameMemoryManager` 的最小 facade。

## 当前能力

当前新增 `NoNameMemoryCompactionService`，支持三类压缩:

- `compact_turn`
  - 输入: turn id、章节索引、地点、角色、目标、文本片段、冲突、未决线索、关系变化
  - 输出: 回合级结构化摘要

- `compact_chapter`
  - 输入: chapter id、章节标题、episodic events、narrative notes
  - 输出: 章节级结构化摘要，聚合角色、地点、目标、冲突、未决线索和人物关系

- `compact_trace`
  - 输入: 一组 `NoNameTrace`
  - 输出: trace 诊断摘要，保留 proposal focus、guardrail/apply/fallback/capability/protocol 诊断信息

## 输出结构

统一输出 `NoNameCompactionSummary`:

- `summaryId`
- `kind`
- `title`
- `summary`
- `chapterIndex`
- `sourceIds`
- `keyEntities`
- `locations`
- `goals`
- `conflicts`
- `unresolvedThreads`
- `relationships`
- `diagnostics`
- `estimatedTokens`
- `createdAt`

这保证压缩结果不是纯文本，而是后续 `context builder`、`note store` 或角色上下文包可以再次读取的结构化对象。

## Memory Manager 接入

`NoNameMemoryManager` 当前提供最小 facade:

- `compact_turn_memory`
- `compact_chapter_memory`
- `compact_trace_memory`
- `upsert_compaction_summary`

其中 `upsert_compaction_summary` 会把摘要转换成 `NoNameNarrativeMemoryItem`，便于后续通过 narrative memory / active notes 读取。

## 当前限制

- 当前压缩是确定性规则式压缩，不调用 LLM。
- 不做深层语义聚类或 embedding 去重。
- 不直接清理原始 memory，只生成可复用摘要。
- 不接管 `tauri_commands`、`noname_runtime` 或 assisted apply 主线。

## 后续建议

下一步可以继续:

1. 让 `context builder` 在预算紧张时优先读取 compaction summary。
2. 与 A2 structured notes 联动，把 unresolved thread / character arc 状态管理得更细。
3. 引入可选 LLM compactor，但保持当前结构化输出契约不变。
